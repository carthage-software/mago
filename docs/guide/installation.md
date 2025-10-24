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
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --version=1.0.0-beta.34
```

Or with `wget`:

```sh
wget -qO- https://carthage.software/mago.sh | bash -s -- --version=1.0.0-beta.34
```

## Manual download

You can always download a pre-compiled binary directly from our GitHub Releases page. This is the **recommended method for Windows** and a reliable fallback for other systems.

1.  Navigate to the **[Mago releases page](https://github.com/carthage-software/mago/releases)**.
2.  Download the appropriate archive for your operating system (e.g., `mago-x86_64-pc-windows-msvc.zip` for Windows).
3.  Unzip the archive.
4.  Place the `mago.exe` (or `mago`) executable in a directory that is part of your system's `PATH` environment variable.

## Package managers

These methods are convenient but may be managed by the community or experience slight publishing delays. If you use Homebrew or Cargo, it is **crucial to run [`mago self-update`](/guide/upgrading)** immediately after installation.

### Composer (PHP project)

To add Mago as a development dependency to your PHP project via Composer:

```sh
composer require --dev "carthage-software/mago:^1.0.0-beta.34"
```

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

## Verify installation

Once installed, you can verify that Mago is working correctly by checking its version:

```sh
mago --version
```
