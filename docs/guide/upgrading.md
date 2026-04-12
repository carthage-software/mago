---
title: "Upgrading Mago"
---

# Upgrading Mago

Keeping **Mago** up-to-date is simple. The `self-update` command handles the entire update process for you, ensuring you always have the latest features, bug fixes, and performance improvements.

This command is the recommended way to update Mago if you installed it using the shell script, Homebrew, Cargo, or by downloading the binary manually.

:::warning
This command is not intended for installations managed by Composer. If you installed Mago as a project dependency, you should use `composer update` to manage its version.
:::

## Usage

### Checking for updates

To see if a new version of Mago is available without installing it, use the `--check` flag. If a newer version exists, the command will print the new version number and exit with a non-zero status code, making it suitable for scripts and CI checks.

```sh
mago self-update --check
```

### Updating to the latest version

The easiest way to update is to run the command with no arguments. It will fetch the latest version, ask for confirmation, and replace the current executable.

```sh
mago self-update
```

To skip the interactive confirmation prompt, use the `--no-confirm` flag.

```sh
mago self-update --no-confirm
```

### Updating to a specific version

You can install a specific version by providing a version tag (e.g., a release number from GitHub) with the `--tag` flag.

```sh
mago self-update --tag 1.0.0-rc.1
```

### Updating to the version pinned in `mago.toml`

:::info Available in Mago 1.20.0+
`--to-project-version` is new in **Mago 1.20.0**. On 1.19.x and earlier, the flag does not exist and `version` in `mago.toml` is ignored; you need to install 1.20.0 (or newer) first, then the flag becomes usable from that point on.
:::

If your project uses [version pinning](/guide/configuration#version-pinning), you can sync the installed Mago binary to whatever the project expects without having to type the version yourself:

```sh
mago self-update --to-project-version
```

This reads the `version` field from `mago.toml` and installs the matching release:

- For an exact pin (`version = "1.19.3"`) this resolves directly to that release tag.
- For a major or minor pin (`version = "1"` or `version = "1.14"`) Mago scans the recent GitHub releases and installs the highest one that satisfies the pin. If you pinned `version = "1"` and 2.0 has already shipped, `--to-project-version` still installs the latest 1.x release without dragging you to 2.0. If you pinned `version = "1.14"` while 1.19.x is out in the wild, it walks back past 1.15/16/17/18/19 and installs the latest 1.14.x.

## Command reference

:::tip
For global options that can be used with any command, see the [Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to specify global options **before** the `self-update` command.
:::

```sh
Usage: mago self-update [OPTIONS]
```

### Options

| Flag, Alias(es)          | Description                                                                                                         |
| :----------------------- | :------------------------------------------------------------------------------------------------------------------ |
| `--check`, `-c`          | Check for updates without installing them. The command will exit with a failure code if a new version is available. |
| `--no-confirm`           | Skip the confirmation prompt before installing an update.                                                           |
| `--tag <VERSION>`        | Update to a specific version tag (e.g., `1.0.0-rc.1`) instead of the latest version. Conflicts with `--to-project-version`. |
| `--to-project-version`   | *Mago 1.20.0+.* Update to whatever version is pinned in `mago.toml`. Fails if no pin is set. See [Version pinning](/guide/configuration#version-pinning). Conflicts with `--tag`. |
| `-h`, `--help`           | Print the help summary for the command.                                                                             |
