---
title: Installation
outline: deep
---

# Installation

Installing **Mago** is a quick process with several options to suit your environment and preferences.

## Shell installer (macOS & Linux)

This is the **recommended method** for most macOS and Linux users. Our script automatically downloads the correct binary for your system and adds it to your path.

#### Using `curl`

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash
```

#### Using `wget`

```sh
wget -qO- https://carthage.software/mago.sh | bash
```

#### Installing a specific version

To install a specific version of Mago, use the `--version=` flag:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --version=1.25.0
```

Or with `wget`:

```sh
wget -qO- https://carthage.software/mago.sh | bash -s -- --version=1.25.0
```

#### Verifying the download

If the [GitHub CLI (`gh`)](https://cli.github.com/) is on your `PATH`, the install script automatically verifies the downloaded archive against our GitHub build attestation before installing anything. No flag required. If `gh` is missing or too old, the script prints a yellow notice and continues without verification.

To make verification mandatory (hard-fail when `gh` is missing, too old, or the attestation does not match), pass `--always-verify`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --always-verify
```

To opt out even when `gh` is available, pass `--no-verify`. See [Verifying release artifacts](#verifying-release-artifacts) for the full picture.

## Manual download

You can always download a pre-compiled binary directly from our GitHub Releases page. This is the **recommended method for Windows** and a reliable fallback for other systems.

1.  Navigate to the **[Mago releases page](https://github.com/carthage-software/mago/releases)**.
2.  Download the appropriate archive for your operating system (e.g., `mago-x86_64-pc-windows-msvc.zip` for Windows).
3.  Unzip the archive.
4.  Place the `mago.exe` (or `mago`) executable in a directory that is part of your system's `PATH` environment variable.

## Docker

The official container image provides a zero-install way to run Mago in any environment. The image is built from `scratch` and weighs only ~26 MB.

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

Available tags include `latest`, exact versions (e.g., `1.25.0`), minor versions (`1.25`), and major versions (`1`). Both `linux/amd64` and `linux/arm64` are supported.

See the [Docker recipe](/recipes/docker) for detailed usage, CI/CD examples, and limitations.

## Package managers

These methods are convenient but may be managed by the community or experience slight publishing delays. If you use Homebrew or Cargo, it is **crucial to run [`mago self-update`](/guide/upgrading)** immediately after installation.

### Composer (PHP project)

To add Mago as a development dependency to your PHP project via Composer:

```sh
composer require --dev "carthage-software/mago:^1.25.0"
```

The Composer package is a thin wrapper: the first `vendor/bin/mago` invocation downloads the matching pre-built binary from the GitHub release and caches it; later calls reuse the cache and make no HTTP requests.

:::tip
If GitHub's anonymous rate limit blocks the download (common on shared CI runners), set `GITHUB_TOKEN` or `GH_TOKEN` on the **first** `mago` call and the wrapper will use it for that request. In GitHub Actions the token isn't exported automatically, so pass it explicitly:

```yaml
- run: mago lint
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

:::

### Homebrew (macOS)

:::warning
The Homebrew formula for Mago is community-managed and often lags significantly behind official releases. This method is **not recommended** unless you follow it with a [self-update](/guide/upgrading).
:::

1.  Install the potentially outdated version from Homebrew:
    ```sh
    brew install mago
    ```
2.  **Immediately run `mago self-update`** to get the latest official version:
    ```sh
    mago self-update
    ```

### Cargo (Rust)

:::info
Publishing to crates.io can sometimes be delayed after a new release.
:::

1.  Install from Crates.io:
    ```sh
    cargo install mago
    ```
2.  Run [`mago self-update`](/guide/upgrading) to ensure you have the absolute latest version:
    ```sh
    mago self-update
    ```

## Verifying release artifacts

Every Mago release archive (per-platform binary tarball, source tarball, source zip, and WASM bundle) is signed at build time via [`actions/attest-build-provenance`](https://github.com/actions/attest-build-provenance). The signature is an [in-toto](https://in-toto.io/) attestation stored on GitHub and tied to the exact workflow run that produced the artifact, so you can prove a download is byte-identical to what came out of `carthage-software/mago`'s release pipeline.

The install script verifies attestations automatically when the GitHub CLI is available. Teams with stricter supply-chain requirements can also make verification mandatory or run it manually.

### Through the install script

The install script picks one of three modes based on the flags you pass:

| Mode             | How to enable                              | Behaviour                                                                                                                                                                                            |
| :--------------- | :----------------------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `auto` (default) | _no flag_                                  | If `gh` is on `PATH` and supports `gh attestation`, the archive is verified before install. If `gh` is missing or too old, the script prints a yellow notice and installs without verifying.         |
| `always`         | `--always-verify`                          | Verification is mandatory. Missing `gh`, a `gh` version too old to support `gh attestation`, or a failed attestation match all abort the install before anything is moved into your `PATH`.         |
| `never`          | `--no-verify`                              | Skip the attestation check even when `gh` is available. Useful when you have already verified out-of-band, or for non-interactive environments where `gh` is present but not authenticated.         |

`--always-verify` and `--no-verify` cannot be combined. The script rejects that combination before doing any work.

When verification runs (auto or always), it executes:

```sh
gh attestation verify <archive> \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml
```

The `--signer-workflow` pin matters: it ties the attestation not just to the right repository but to the exact release workflow file. A leaked GitHub Actions token that could trigger a *different* workflow inside the same repository would still fail verification.

If the attestation does not match, the script copies the unverified archive to the current working directory as `<file>.unverified.tar.gz` (so it survives the temp-dir cleanup and you can inspect it forensically), prints a red error, and exits before extraction. Nothing is moved into your `PATH`.

Examples:

```sh
# Auto: verify if `gh` is available, otherwise install without verifying.
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash

# Mandatory verification (CI, security-conscious installs).
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh \
  | bash -s -- --always-verify

# Combine with other flags as usual.
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh \
  | bash -s -- --always-verify --version=1.25.0 --install-dir=/opt/bin

# Explicit opt-out.
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --no-verify
```

Pre-requisites for verification: install a recent [GitHub CLI](https://cli.github.com/) that ships the `gh attestation` subcommand. The verify call reads from the public attestations API, so no `gh auth` is required.

### Manual verification (downloaded from GitHub Releases)

If you fetched an archive directly from the [releases page](https://github.com/carthage-software/mago/releases) (for example, on Windows where the install script does not run, or because you want to keep the archive around), you can verify it yourself before extracting:

```sh
VERSION=1.25.0
TARGET=x86_64-unknown-linux-gnu                      # match your platform
ASSET=mago-${VERSION}-${TARGET}.tar.gz

gh release download "$VERSION" --repo carthage-software/mago --pattern "$ASSET"
gh attestation verify "$ASSET" \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml

tar -xzf "$ASSET"
sudo mv "mago-${VERSION}-${TARGET}/mago" /usr/local/bin/
```

A successful run prints `✓ Verification succeeded!` along with the workflow run that produced the archive. A failure exits non-zero and explains why.

:::tip
The attestation is bound to the **archive**, not to the binary inside it. If you only kept the extracted binary, you cannot verify it directly. Re-download the archive, verify it, and compare its inner binary's `sha256sum` to the one already on your system.
:::

### Pinning the install script

The shell installer URL above (`https://carthage.software/mago.sh`) is a redirect to the latest version of [`scripts/install.sh`](https://github.com/carthage-software/mago/blob/main/scripts/install.sh) in the `main` branch. Future revisions to that file are picked up automatically, which is convenient but means a future change in installer behaviour also lands automatically.

For a stricter supply chain, pin the installer to a specific commit you have reviewed. Pick a commit on `main` whose `scripts/install.sh` you have read end-to-end, then fetch the script from that exact SHA:

```sh
# the commit that shipped --always-verify; replace with a newer one you have reviewed
COMMIT=cd4cf4dfdbc72bd028ad26d11bcc815a49e27e9a
curl --proto '=https' --tlsv1.2 -sSf \
  "https://raw.githubusercontent.com/carthage-software/mago/${COMMIT}/scripts/install.sh" \
  | bash -s -- --always-verify
```

GitHub rewrites neither the file nor the SHA after the fact, so the bytes you reviewed are the bytes you run. Updating the pin later is a deliberate decision: read the new commit's `install.sh`, then bump `$COMMIT`.

## Verify installation

Once installed, you can verify that Mago is working correctly by checking its version:

```sh
mago --version
```
