# Termux (Android / ARM64) dev environment

Tooling for developing Mycelium on a phone under **Termux**. Not part of Mycelium's
product — it's the environment that runs the LLM harness, the KC-2 experiment, and the
on-device Rust builds (`tools/llm-harness/`, `experiments/`). Captured here so it is
version-controlled and recoverable (an earlier ad-hoc copy was lost to a `clang` reinstall).

## `cc-termux-bootstrap.sh` — Claude Code on Android

Termux can't run the official `claude` binary directly (wrong-arch / exec-format errors),
so this provisions a real **glibc Ubuntu** via `proot-distro` on internal storage and
installs Claude Code inside it, where the official installer and background auto-update
work. A thin Termux-side launcher enters the container and re-asserts the environment on
every run.

```sh
bash tools/termux/cc-termux-bootstrap.sh        # do NOT `source` it
```

It installs a launcher (default name **`claude`**) with:

| command            | what it does |
|--------------------|--------------|
| `claude`           | launch Claude Code in the default workspace (`/home/<user>/work`) |
| `claude work PATH` | launch in a path inside the container |
| `claude sd`        | launch with the workspace on the SD card (bulk, non-exec data only) |
| `claude update`    | controlled update (pre → update → post hooks, with backup + validate) |
| `claude doctor`    | validate the environment |
| `claude shell`     | drop into the Ubuntu shell |

Want the old `cc` muscle memory? Use a **shell alias**, never a file:
`echo 'alias cc=claude' >> ~/.bashrc`.

### Tunables (export before running)

`CC_DISTRO` (default `ubuntu`), `CC_DEV_USER` (`dev`), `CC_LAUNCHER` (`claude`),
`CC_SD_SRC`, `CC_SD_GUEST`, `CC_WORK_GUEST`, `CC_SUDO_MODE` (`nopasswd` | `password`).

### Idempotency

Safe to re-run. The container is reused if already enterable, the dev user is created
only if absent, packages/config/hooks are re-asserted, and Claude is installed only if
missing. Re-running repairs drift; it never duplicates.

### Secrets

- **Nothing sensitive is stored** in the script or this repo. Claude auth is interactive
  at first run (browser OAuth or an API key you paste) and lives in `~/.claude` inside the
  container.
- The dev user defaults to **passwordless sudo** — it's a single-user proot sandbox, not a
  privilege boundary. For a password instead, run with `CC_SUDO_MODE=password`: you're
  **prompted** (no echo, with confirmation), and the secret is piped to the container over
  stdin — never in argv, the environment, on disk, or committed.

### ⚠️ Never name the launcher after a compiler

`CC_LAUNCHER` must not be `cc`/`clang`/`gcc`/`clang++`/… — those names *are* the C/C++
compiler on `PATH`. An earlier version defaulted to `cc`, which overwrote
`$PREFIX/bin/cc → clang` and broke **every** native build (`cargo` build scripts failed
with `Unknown command '…/symbols.o'. Try: cc help`) until `pkg install --reinstall clang`.
The script now defaults to `claude` and refuses compiler/toolchain names outright.

## Building Mycelium's Rust on Termux

See `experiments/README.md` → "Termux / Android (ARM64) build notes" for the `myc-check`
build prerequisites (the impostor-`cc` failure mode above, and the missing-library case).
