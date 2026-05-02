+++
title = "Upgrading"
description = "Keep Mago up to date with the self-update command, including pinning to a specific version or syncing with the version field in mago.toml."
nav_order = 80
nav_section = "Guide"
+++
# Upgrading

`mago self-update` replaces the running binary with a newer release. Use it for installs that came from the shell script, Homebrew, Cargo, or a manual download.

> Composer installs are different. The Composer wrapper pins a binary that matches the Composer package version, so you upgrade Mago with `composer update` rather than `self-update`.

## Common flows

Check for updates without installing:

```sh
mago self-update --check
```

The command prints the new version (if any) and exits non-zero when one is available, which makes it scriptable in CI.

Update to the latest release:

```sh
mago self-update                  # interactive confirmation
mago self-update --no-confirm     # skip the prompt
```

Pin a specific version:

```sh
mago self-update --tag 1.25.2
```

## Sync with the project's version pin

If your `mago.toml` uses [version pinning](/guide/configuration/#version-pinning), you can sync the installed binary to whatever the project expects without typing the version yourself:

```sh
mago self-update --to-project-version
```

For an exact pin (`version = "1.25.2"`), this resolves directly to that release tag. For a major or minor pin, Mago scans recent GitHub releases and installs the highest one that still satisfies the pin. So `version = "1"` with 2.0 already shipped still installs the latest 1.x release. `version = "1.14"` with 1.19.x in the wild walks back to the latest 1.14.x.

The command fails only if no published release satisfies the pin at all.

## Reference

```sh
Usage: mago self-update [OPTIONS]
```

| Flag | Description |
| :--- | :--- |
| `--check`, `-c` | Check for updates without installing. Exits non-zero when an update is available. |
| `--no-confirm` | Skip the interactive confirmation prompt. |
| `--tag <VERSION>` | Install a specific release tag instead of the latest. Mutually exclusive with `--to-project-version`. |
| `--to-project-version` | Install whatever the project's `version` pin demands. Fails if no pin is set. Mutually exclusive with `--tag`. |
| `-h`, `--help` | Print help and exit. |

Global flags must come before `self-update`. See the [CLI overview](/fundamentals/command-line-interface/) for the full list.
