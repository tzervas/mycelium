# Reproducible toolchain distribution (M-734; E16-1)

A pinned, **content-addressed** install path for the Mycelium toolchain: an artifact is verified
against a committed content-address pin *before* it is copied into place, so an install is
reproducible and a tampered/divergent artifact is a never-silent error (G2).

## Use

```sh
# Pin an artifact directory (writes scripts/dist/pins by default):
scripts/dist/install.sh --artifact <dir> --update-pins

# Verify an artifact against the pin (no copy):
scripts/dist/install.sh --artifact <dir> --verify-only

# Verify, then install into a prefix (idempotent / byte-identical on re-run):
scripts/dist/install.sh --artifact <dir> --prefix <dest>

# Self-test the mechanism end-to-end:
just dist-verify
```

## How it works

The pin is a **self-describing content address** (`<algo>:<hex>`): `blake3` (ADR-003) when `b3sum`
is available, else `sha256`. The artifact root is the hash of a canonical, sorted per-file manifest
(`relpath \t <algo>:<hex>`) — a Merkle-style root mirroring the spore content-address. Verification
checks every pinned file **and** the root (so an *added* file is caught too).

**Strict, never skip-graceful (DoD):** if no supported hasher is present, the script **hard-fails**
with instructions rather than silently skipping the integrity check. A pin that names a specific
algorithm requires that algorithm's hasher to be present.

Exit codes: `0` ok · `5` integrity mismatch · `64` usage · `66` I/O · `69` no hasher available.

## Honest scope (VR-5)

This ships the **pin / verify / install mechanism**, fully reproducible for any given artifact
directory and proven by `just dist-verify` (byte-identical re-install; tamper and missing-file
detection). Pinning the *actual released compiler binaries* additionally needs a **reproducible
release build** so two machines produce byte-identical binaries — that build is deferred and is
**not** claimed here. We do not commit a pin of locally-built (non-reproducible) binaries; the
committed mechanism is what is durable today. This gate does not skip-gracefully — see above.
