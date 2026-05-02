+++
title = "Installation"
description = "Install Mago via the shell installer, a manual download, Docker, or your language's package manager."
nav_order = 20
nav_section = "Guide"
+++
# Installation

Mago ships as a single static binary. Pick whichever installation route suits your environment.

## Shell installer (macOS, Linux)

The recommended path on macOS and Linux. The script detects your platform, fetches the matching release archive, and drops the binary into your PATH.

With `curl`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash
```

With `wget`:

```sh
wget -qO- https://carthage.software/mago.sh | bash
```

### Pin a specific version

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --version=1.25.2
```

The same syntax works with `wget`.

### Verify the download

If the [GitHub CLI](https://cli.github.com/) is on your PATH, the installer verifies the archive against Mago's GitHub build attestation before unpacking it. No flag required. If `gh` is missing or too old, the script prints a notice and continues without verification.

To make verification mandatory, pass `--always-verify`. The installer aborts before touching your PATH if `gh` is unavailable, too old, or the attestation does not match.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --always-verify
```

To opt out entirely, pass `--no-verify`. The two flags are mutually exclusive.

## Manual download

The recommended path on Windows and a fine fallback on any system without `bash`.

1. Open the [releases page](https://github.com/carthage-software/mago/releases).
2. Download the archive for your operating system. The naming follows `mago-<version>-<target>.tar.gz` (or `.zip` on Windows).
3. Extract the archive and place the binary somewhere on your PATH.

If you keep the archive around, you can verify it yourself before extracting.

```sh
VERSION=1.25.2
TARGET=x86_64-unknown-linux-gnu  # adjust for your platform
ASSET=mago-${VERSION}-${TARGET}.tar.gz

gh release download "$VERSION" --repo carthage-software/mago --pattern "$ASSET"
gh attestation verify "$ASSET" \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml

tar -xzf "$ASSET"
sudo mv "mago-${VERSION}-${TARGET}/mago" /usr/local/bin/
```

A successful verification prints `Verification succeeded!` and the workflow run that produced the archive.

The attestation is bound to the archive, not to the extracted binary. If you only kept the binary you cannot verify it directly. Re-download the archive, verify it, and compare the inner binary's `sha256sum` to the one already on your system.

## Docker

The official image is built from `scratch` and weighs roughly 26 MB. It runs anywhere Docker does, supports `linux/amd64` and `linux/arm64`, and needs no host PHP runtime.

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

Tags include `latest`, exact versions, and progressively looser pins (for example `1.25.2`, `1.25`, `1`). The [Docker recipe](/recipes/docker/) covers CI examples and the limitations to be aware of.

## Package managers

These routes are convenient but rely on external publishing schedules that often lag the GitHub release. After installing through any of them, run [`mago self-update`](/guide/upgrading/) to pull the latest official binary.

### Composer

For PHP projects:

```sh
composer require --dev "carthage-software/mago:^1.25.2"
```

The Composer package is a thin wrapper. The first call to `vendor/bin/mago` downloads the matching pre-built binary from the GitHub release and caches it. Subsequent calls reuse the cache and make no network requests.

If GitHub's anonymous rate limit blocks the first download (common on shared CI runners) set `GITHUB_TOKEN` or `GH_TOKEN` for that one invocation. In GitHub Actions the token is not exported automatically, so pass it explicitly:

```yaml
- run: vendor/bin/mago lint
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Homebrew

The community-maintained formula often lags the official release. Install it, then run `mago self-update` immediately.

```sh
brew install mago
mago self-update
```

### Cargo

Crates.io publishing can lag a few hours behind a release. Same pattern as Homebrew.

```sh
cargo install mago
mago self-update
```

## Verifying releases in detail

Every release archive (per-platform tarball, source tarball, source zip, and WASM bundle) is signed at build time via [`actions/attest-build-provenance`](https://github.com/actions/attest-build-provenance). The signature is an [in-toto](https://in-toto.io/) attestation stored on GitHub and tied to the workflow run that produced the artifact, so a verified download is provably byte-identical to what came out of Mago's release pipeline.

The shell installer chooses one of three modes based on the flags you pass.

| Mode | Flag | Behaviour |
| :--- | :--- | :--- |
| `auto` | none | Verifies if `gh` is available; otherwise installs without verifying. |
| `always` | `--always-verify` | Verification is mandatory. Missing or too-old `gh`, or a failed match, aborts the install. |
| `never` | `--no-verify` | Skips the check even when `gh` is available. |

When verification runs, the installer executes:

```sh
gh attestation verify <archive> \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml
```

The `--signer-workflow` pin matters. It binds the attestation to the exact release workflow file. A leaked GitHub Actions token that could trigger a different workflow inside the same repository would still fail verification.

If the check fails, the script copies the unverified archive to your current working directory as `<file>.unverified.tar.gz` (so it survives the temp-dir cleanup and you can inspect it forensically), prints a red error, and exits before extraction. Nothing reaches your PATH.

The verify call reads from the public attestations API, so no `gh auth` is required. You only need a recent `gh` that includes the `gh attestation` subcommand.

### Pinning the install script

`https://carthage.software/mago.sh` redirects to [`scripts/install.sh`](https://github.com/carthage-software/mago/blob/main/scripts/install.sh) on the `main` branch. Future revisions are picked up automatically, which is convenient but also means future installer changes land without warning.

For stricter supply-chain hygiene, pin the script to a specific commit you have reviewed:

```sh
COMMIT=cd4cf4dfdbc72bd028ad26d11bcc815a49e27e9a  # replace with a commit you have read
curl --proto '=https' --tlsv1.2 -sSf \
  "https://raw.githubusercontent.com/carthage-software/mago/${COMMIT}/scripts/install.sh" \
  | bash -s -- --always-verify
```

GitHub does not rewrite files at a given SHA, so the bytes you reviewed are the bytes you run. Updating the pin is a deliberate act: read the new commit, then bump `COMMIT`.

## Verify the install

```sh
mago --version
```

If that prints a version, you are ready to [run Mago against your code](/guide/getting-started/).
