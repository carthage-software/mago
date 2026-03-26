---
title: Analyzer command reference
outline: deep
---

# Command reference

The `mago analyze` command is the entry point for running Mago's static type checker.

:::tip
For global options that can be used with any command, see the [Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to specify global options **before** the `analyze` command.
:::

```sh
Usage: mago analyze [OPTIONS] [PATHS]...
```

:::tip
`mago analyse` is a convenient alias for `mago analyze`. Both can be used interchangeably.
:::

## Arguments

### `[PATHS]...`

Optional. A list of specific files or directories to analyze. If you provide paths here, they will be used instead of the `paths` defined in your `mago.toml` configuration.

## Options

| Flag, Alias(es) | Description                                                                          |
| :-------------- | :----------------------------------------------------------------------------------- |
| `--no-stubs`    | Analyze the project without loading the built-in PHP stubs for the standard library. |
| `--staged`      | Only analyze files that are staged in git. Designed for pre-commit hooks. Fails if not in a git repository. |
| `--watch`       | Enable watch mode for continuous analysis. Re-runs analysis when files change. See [Watch Mode](#watch-mode) below. |
| `--list-codes`  | List all available analyzer issue codes in JSON format.                               |
| `--help`, `-h`  | Print the help summary for the command.                                              |

### Shared Reporting and Fixing Options

The `analyze` command shares a common set of options with other Mago tools for reporting, fixing, and baseline management.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)

## Watch Mode

When `--watch` is enabled, the analyzer continuously monitors your workspace for changes and automatically re-runs analysis whenever PHP files are modified, created, or deleted.

```sh
mago analyze --watch
```

### Automatic Restart

In addition to PHP file changes, the analyzer watches for changes to project configuration files. When any of the following files change, the analyzer **automatically restarts** with the reloaded configuration:

- **Configuration file** — the `mago.toml` (or whichever config file was loaded)
- **Baseline file** — if configured via `analyzer.baseline`
- **`composer.json`** and **`composer.lock`**

This means you can edit your `mago.toml` (e.g., add an ignore rule or change a setting) and the analyzer will pick up the changes without needing to manually restart.

:::tip
If no configuration file exists when watch mode starts, the analyzer watches for the creation of any supported config file (`mago.toml`, `mago.yaml`, `mago.json`, etc.) and restarts when one appears.
:::

Press **Ctrl+C** to stop watching.
