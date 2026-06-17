#!/usr/bin/env python3
"""Idempotent, cross-platform commit-signing reconciler for Mycelium (pure Python + git/gpg/gh).

Signed commits are a standard git/GitHub convention; this wires them up the same honest way the
PM reconciler wires the project — **nondestructive, idempotent, never-silent**, and identical on
Linux/macOS and Windows (it shells to `git`/`gpg`/`gh`, which resolve `.exe` via PATHEXT; no bash,
no jq).

    python tools/github/git-signing-sync.py              # SANITY CHECK only (read-only, zero writes)
    python tools/github/git-signing-sync.py --setup      # configure signing (prompts name/email/comment)
    python tools/github/git-signing-sync.py --setup --new-key   # force a FRESH key (rotation)
    python tools/github/git-signing-sync.py --setup --dry-run   # preview the setup plan
    python tools/github/git-signing-sync.py --self-test  # offline check of the pure decision logic

Behaviour (the house rules in CLAUDE.md):

  * **Default = sanity check.** With no flags it only DETECTS and REPORTS — are git/gpg/gh
    installed, is a git identity set, is there a signing key for the email, is commit signing
    wired? It writes nothing. If something is missing it prompts (interactively) to run setup, or
    prints the exact remediation (non-interactive). If already set up it says so and exits 0 — you
    can proceed with the maintenance/automation workflow.
  * **`--setup`/`--init` is the explicit, opt-in trigger.** It prompts for the GPG user-id fields
    (name / email / comment), then **reuses an existing key** and only **generates** one when none
    exists (first-time) or when **`--new-key`** forces a rotation. An existing key is **never
    replaced without `--new-key`** (truly nondestructive). Git config is set **create-if-absent /
    update-on-drift** — an already-correct config is a no-op.
  * **never-silent (G2)** — every detection result and every change is printed; `--dry-run`
    previews and writes nothing. A missing component is an explicit, actionable message, never a
    silent skip.
  * **secret-free** — the **private** key never leaves the machine; only the **public** key is
    uploaded (and only with `--upload`, via `gh gpg-key add`). No token is read or stored here.

No third-party dependency (KC-3): standard library + the `git`/`gpg`/`gh` CLIs only.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys

# ─────────────────────────────────────────────────────────────────────────────────────────────
# tiny logger (color only on a tty)
# ─────────────────────────────────────────────────────────────────────────────────────────────
if sys.stdout.isatty():
    C_G, C_Y, C_R, C_D, C_0 = "\033[32m", "\033[33m", "\033[31m", "\033[2m", "\033[0m"
else:
    C_G = C_Y = C_R = C_D = C_0 = ""


def step(msg):
    print(f"\n{C_D}== {msg} =={C_0}")


def ok(msg):
    print(f"  {C_G}ok{C_0}   {msg}")


def info(msg):
    print(f"  {C_D}..{C_0}   {msg}")


def warn(msg):
    print(f"  {C_Y}warn{C_0} {msg}")


def bad(msg):
    print(f"  {C_R}miss{C_0} {msg}")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# plumbing (cross-platform: list argv, never shell=True)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def run(args, *, check=True, input_text=None):
    """Run a CLI command; return CompletedProcess. Never spawns a shell (Windows-safe)."""
    return subprocess.run(
        list(args), check=check, text=True, input=input_text, capture_output=True
    )


def git_config_get(key):
    """Return the global git config value for ``key`` or '' when unset (rc 1 is 'unset', not error)."""
    proc = run(["git", "config", "--global", "--get", key], check=False)
    return proc.stdout.strip() if proc.returncode == 0 else ""


def gpg_keyid_for(email):
    """Return the long key-id of an existing secret signing key for ``email``, or '' if none."""
    if not email or not shutil.which("gpg"):
        return ""
    proc = run(
        ["gpg", "--list-secret-keys", "--keyid-format=long", "--with-colons", email],
        check=False,
    )
    if proc.returncode != 0:
        return ""
    for line in proc.stdout.splitlines():
        parts = line.split(":")
        if parts and parts[0] == "sec":  # field 5 (index 4) is the long key id
            return parts[4]
    return ""


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Detection (read-only snapshot of the local signing state)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def detect_state(email_override=None):
    """Snapshot the local commit-signing state without changing anything."""
    has_git = bool(shutil.which("git"))
    name = git_config_get("user.name") if has_git else ""
    email = email_override or (git_config_get("user.email") if has_git else "")
    return {
        "git": has_git,
        "gpg": bool(shutil.which("gpg")),
        "gh": bool(shutil.which("gh")),
        "name": name,
        "email": email,
        "keyid": gpg_keyid_for(email),
        "cfg": {
            "user.signingkey": git_config_get("user.signingkey") if has_git else "",
            "commit.gpgsign": git_config_get("commit.gpgsign") if has_git else "",
            "tag.gpgsign": git_config_get("tag.gpgsign") if has_git else "",
            "gpg.format": git_config_get("gpg.format") if has_git else "",
            "gpg.program": git_config_get("gpg.program") if has_git else "",
        },
    }


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Pure decision logic (no I/O — exercised offline by --self-test)
# ─────────────────────────────────────────────────────────────────────────────────────────────
def desired_config(keyid, gpg_program):
    """The git-config a GPG-signing setup wants for ``keyid`` (used to compute drift)."""
    return {
        "user.signingkey": keyid,
        "commit.gpgsign": "true",
        "tag.gpgsign": "true",
        "gpg.format": "openpgp",
        "gpg.program": gpg_program,
    }


def config_drift(desired, current):
    """Return only the {key: value} that must change (create-if-absent + update-on-drift)."""
    return {k: v for k, v in desired.items() if v and current.get(k, "") != v}


def plan_signing(state, *, new_key, gpg_program="gpg"):
    """Decide the signing actions for a `--setup` run. Pure: state + flags -> plan.

    Honest + nondestructive: generate ONLY when no key exists or ``--new-key`` forces a rotation;
    otherwise reuse. An existing **ssh**-format signing setup is left untouched (a different,
    deliberate choice) unless ``--new-key`` is given. ``blocked`` is non-empty when a hard
    prerequisite (git/gpg) is missing — the caller must stop, never half-apply.
    """
    plan = {
        "key_action": "none",
        "reason": "",
        "keyid": "",
        "config_writes": {},
        "flags": [],
        "blocked": [],
    }
    if not state["git"]:
        plan["blocked"].append("git is not installed")
    if not state["gpg"]:
        plan["blocked"].append("gpg is not installed")
    if plan["blocked"]:
        return plan

    cfg = state["cfg"]
    if cfg.get("gpg.format") == "ssh" and cfg.get("user.signingkey") and not new_key:
        # Already signing with SSH — nondestructive: do not clobber a deliberate choice.
        plan["flags"].append(
            "git is already configured for SSH commit signing — leaving as-is "
            "(pass --new-key to switch to a fresh GPG key)"
        )
        return plan

    if state["keyid"] and not new_key:
        plan["key_action"], plan["keyid"] = "reuse", state["keyid"]
        plan["reason"] = "an existing key for this email is reused (nondestructive)"
    elif state["keyid"] and new_key:
        plan["key_action"] = "generate"
        plan["reason"] = (
            "rotation forced by --new-key (a fresh key replaces the wiring)"
        )
    else:
        plan["key_action"] = "generate"
        plan["reason"] = "no signing key exists for this email (first-time setup)"

    # For reuse we can compute the config drift now; for generate the key-id is only known
    # after generation, so the caller fills config_writes once it has the new id.
    if plan["key_action"] == "reuse":
        plan["config_writes"] = config_drift(
            desired_config(plan["keyid"], gpg_program), cfg
        )
    return plan


def signing_status(state):
    """Classify the read-only state as 'ready' | 'partial' | 'absent' with a reason."""
    if not state["git"] or not state["gpg"]:
        missing = [t for t in ("git", "gpg") if not state[t]]
        return "absent", f"missing tool(s): {', '.join(missing)}"
    cfg = state["cfg"]
    signing_on = cfg.get("commit.gpgsign") == "true"
    has_key = bool(cfg.get("user.signingkey"))
    if cfg.get("gpg.format") == "ssh" and has_key and signing_on:
        return "ready", "SSH commit signing is configured"
    if has_key and signing_on and state["keyid"]:
        return (
            "ready",
            f"GPG commit signing is configured (key {cfg['user.signingkey']})",
        )
    if state["keyid"] and not signing_on:
        return "partial", "a key exists but commit signing is not wired (run --setup)"
    return "absent", "no signing key / signing not configured (run --setup)"


# ─────────────────────────────────────────────────────────────────────────────────────────────
# I/O: prompts + apply
# ─────────────────────────────────────────────────────────────────────────────────────────────
def prompt(label, default=""):
    """Prompt with an optional default; return the entry (or the default on empty input)."""
    suffix = f" [{default}]" if default else ""
    try:
        entry = input(f"  {label}{suffix}: ").strip()
    except EOFError:
        entry = ""
    return entry or default


def resolve_uid(state, args):
    """Resolve the GPG user-id fields (name/email/comment), prompting only when interactive."""
    name, email, comment = args.name, args.email, args.comment
    interactive = sys.stdin.isatty() and not args.no_prompt
    if interactive:
        name = name or prompt("real name", state["name"])
        email = email or prompt("email", state["email"])
        if comment is None:
            comment = prompt("comment (optional)")
    else:
        name = name or state["name"]
        email = email or state["email"]
        comment = comment or ""
    if not name or not email:
        sys.exit(
            "ERROR: a name and email are required. Provide --name/--email (or run "
            "interactively, or set git config user.name/user.email)."
        )
    return name, email, comment or ""


def gpg_generate(name, email, comment, *, passphrase):
    """Generate an ed25519 signing key for the user-id; return its long key-id."""
    uid = f"{name} ({comment}) <{email}>" if comment else f"{name} <{email}>"
    if passphrase:
        info("you will be prompted (pinentry) for a passphrase to protect the key")
        run(["gpg", "--quick-generate-key", uid, "ed25519", "sign", "2y"], check=True)
    else:
        warn("generating an UNPROTECTED key (--no-passphrase)")
        run(
            [
                "gpg",
                "--batch",
                "--pinentry-mode",
                "loopback",
                "--passphrase",
                "",
                "--quick-generate-key",
                uid,
                "ed25519",
                "sign",
                "2y",
            ],
            check=True,
        )
    keyid = gpg_keyid_for(email)
    if not keyid:
        sys.exit("ERROR: GPG key generation reported success but no key was found.")
    return keyid


def apply_config(writes, *, dry_run):
    for key, value in writes.items():
        if dry_run:
            print(f"   ~ would set git config --global {key} {value}")
        else:
            run(["git", "config", "--global", key, value], check=True)
            print(f"   ~ git config --global {key} {value}")


def maybe_upload(keyid, *, want_upload, dry_run):
    if not want_upload:
        info(
            f"public key NOT uploaded (pass --upload to push it). Export: gpg --armor --export {keyid}"
        )
        return
    if not shutil.which("gh"):
        warn(
            "gh not found — cannot upload the public key (install gh, or upload it in GitHub settings)"
        )
        return
    if dry_run:
        print(f"   ~ would upload public key {keyid} via `gh gpg-key add -`")
        return
    pub = run(["gpg", "--armor", "--export", keyid], check=True).stdout
    res = run(["gh", "gpg-key", "add", "-"], check=False, input_text=pub)
    if res.returncode == 0:
        ok(f"public key {keyid} uploaded to GitHub")
    else:
        warn(
            "public key not uploaded (already present, or scope admin:gpg_key not granted)"
        )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Modes
# ─────────────────────────────────────────────────────────────────────────────────────────────
def report(state):
    """Read-only sanity check. Returns the status string ('ready'|'partial'|'absent')."""
    step("commit-signing sanity check (read-only)")
    (ok if state["git"] else bad)(
        f"git    {'found' if state['git'] else 'NOT installed'}"
    )
    (ok if state["gpg"] else bad)(
        f"gpg    {'found' if state['gpg'] else 'NOT installed'}"
    )
    (ok if state["gh"] else warn)(
        f"gh     {'found' if state['gh'] else 'not installed (only needed to upload the public key)'}"
    )
    if state["name"] or state["email"]:
        ok(f"identity  {state['name']} <{state['email']}>")
    else:
        bad("identity  git user.name / user.email not set")
    if state["keyid"]:
        ok(f"key       {state['keyid']} (secret key present for {state['email']})")
    else:
        info("key       none found for this email")
    status, why = signing_status(state)
    color = {"ready": ok, "partial": warn, "absent": bad}[status]
    color(f"signing   {status.upper()} — {why}")
    return status


def do_setup(state, args):
    """The explicit, opt-in setup path: ensure a key + wire config, idempotently."""
    step("commit-signing setup")
    plan = plan_signing(state, new_key=args.new_key)
    for flag in plan["flags"]:
        warn(flag)
    if plan["blocked"]:
        for blk in plan["blocked"]:
            bad(blk)
        sys.exit(
            "ERROR: install the missing component(s) first "
            "(Linux: apt/pkg install git gnupg ; Windows: winget install Git.Git GnuPG.GnuPG)."
        )
    if plan["key_action"] == "none":  # e.g. SSH already configured — nothing to do
        ok("nothing to do (signing already configured); nondestructive no-op")
        return

    name, email, comment = resolve_uid(state, args)
    info(f"plan: {plan['reason']}")

    if plan["key_action"] == "reuse":
        keyid = plan["keyid"]
        ok(f"reusing existing key {keyid}")
        writes = plan["config_writes"]
    else:  # generate
        if args.dry_run:
            print(f"   + would generate an ed25519 signing key for {name} <{email}>")
            keyid = "<new-key-id>"
        else:
            keyid = gpg_generate(
                name, email, comment, passphrase=not args.no_passphrase
            )
            ok(f"generated key {keyid}")
        writes = config_drift(
            desired_config(keyid, shutil.which("gpg") or "gpg"), state["cfg"]
        )

    if writes:
        apply_config(writes, dry_run=args.dry_run)
    else:
        ok("git signing config already correct (no changes)")
    maybe_upload(keyid, want_upload=args.upload, dry_run=args.dry_run)
    print()
    ok(
        "signing setup complete"
        + (" (dry-run — nothing written)" if args.dry_run else "")
    )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Offline self-test of the pure logic
# ─────────────────────────────────────────────────────────────────────────────────────────────
def self_test():
    base = {
        "git": True,
        "gpg": True,
        "gh": True,
        "name": "Dev",
        "email": "d@e.x",
        "keyid": "",
        "cfg": {
            "user.signingkey": "",
            "commit.gpgsign": "",
            "tag.gpgsign": "",
            "gpg.format": "",
            "gpg.program": "",
        },
    }
    # No key, no --new-key, under --setup → generate (first-time is the explicit trigger).
    p = plan_signing(base, new_key=False)
    assert p["key_action"] == "generate" and "first-time" in p["reason"], p

    # Existing key, no --new-key → REUSE (nondestructive); config drift computed.
    have = dict(base, keyid="ABCD1234")
    p = plan_signing(have, new_key=False, gpg_program="/usr/bin/gpg")
    assert p["key_action"] == "reuse" and p["keyid"] == "ABCD1234", p
    assert p["config_writes"]["user.signingkey"] == "ABCD1234"
    assert p["config_writes"]["commit.gpgsign"] == "true"

    # Existing key already fully wired → reuse with ZERO config writes (idempotent).
    wired = dict(
        base,
        keyid="ABCD1234",
        cfg={
            "user.signingkey": "ABCD1234",
            "commit.gpgsign": "true",
            "tag.gpgsign": "true",
            "gpg.format": "openpgp",
            "gpg.program": "/usr/bin/gpg",
        },
    )
    p = plan_signing(wired, new_key=False, gpg_program="/usr/bin/gpg")
    assert p["key_action"] == "reuse" and p["config_writes"] == {}, p

    # Existing key + --new-key → generate (rotation), never silent about replacing.
    p = plan_signing(have, new_key=True)
    assert p["key_action"] == "generate" and "rotation" in p["reason"], p

    # SSH signing already configured + no --new-key → left untouched (nondestructive).
    ssh = dict(
        base,
        cfg=dict(base["cfg"], **{"gpg.format": "ssh", "user.signingkey": "ssh-key"}),
    )
    p = plan_signing(ssh, new_key=False)
    assert p["key_action"] == "none" and any("SSH" in f for f in p["flags"]), p

    # Missing gpg → blocked (caller must stop; never half-apply).
    p = plan_signing(dict(base, gpg=False), new_key=False)
    assert p["blocked"] and p["key_action"] == "none", p

    # config_drift writes only what differs.
    assert config_drift({"a": "1", "b": "2"}, {"a": "1", "b": ""}) == {"b": "2"}

    # status classification.
    assert signing_status(wired)[0] == "ready"
    assert signing_status(dict(base, keyid="X"))[0] in ("partial", "absent")
    assert signing_status(dict(base, gpg=False))[0] == "absent"

    print("self-test OK: plan_signing, config_drift, signing_status")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# CLI
# ─────────────────────────────────────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(
        description="Idempotent, cross-platform commit-signing reconciler (sanity check by default; "
        "--setup to configure)."
    )
    parser.add_argument(
        "--setup",
        "--init",
        dest="setup",
        action="store_true",
        help="configure commit signing (prompts name/email/comment)",
    )
    parser.add_argument(
        "--new-key",
        "--rotate",
        dest="new_key",
        action="store_true",
        help="force a FRESH key even if one exists (rotation) — the only path that replaces a key",
    )
    parser.add_argument("--name", help="GPG user-id real name (skip the prompt)")
    parser.add_argument("--email", help="GPG user-id email (skip the prompt)")
    parser.add_argument(
        "--comment", help="GPG user-id comment (skip the prompt; '' for none)"
    )
    parser.add_argument(
        "--no-passphrase",
        action="store_true",
        help="generate the key WITHOUT a passphrase (less secure; warned)",
    )
    parser.add_argument(
        "--upload",
        action="store_true",
        help="upload the PUBLIC key to GitHub via `gh gpg-key add`",
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="preview the setup plan; write nothing"
    )
    parser.add_argument(
        "--no-prompt",
        action="store_true",
        help="never prompt (non-interactive); require --name/--email or git config",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run the offline decision-logic check and exit",
    )
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return

    state = detect_state(email_override=args.email)

    if not args.setup:
        status = report(state)
        if status == "ready":
            print()
            ok("signing ready — proceed with the maintenance/automation workflow")
            return
        # Not ready: offer setup interactively, else print the remediation (never silent).
        if sys.stdin.isatty() and not args.no_prompt:
            if prompt("run signing setup now? (y/N)").lower().startswith("y"):
                do_setup(detect_state(email_override=args.email), args)
                return
        print()
        info(
            "run `python tools/github/git-signing-sync.py --setup` to configure signing"
        )
        sys.exit(1)

    do_setup(state, args)


if __name__ == "__main__":
    main()
