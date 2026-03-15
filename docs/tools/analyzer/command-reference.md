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

| Flag, Alias(es) | Description                                                                                                                                                                   |
|:----------------|:------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--no-stubs`    | Analyze the project without loading the built-in PHP stubs for the standard library.                                                                                          |
| `--staged`      | Only analyze files that are staged in git. Designed for pre-commit hooks. Fails if not in a git repository.                                                                   |
| `--stdin-input` | Read file content from stdin and use the single path argument for baseline and reporting. Intended for editor integrations (e.g. unsaved buffers). Requires exactly one path. |
| `--watch`       | Enable watch mode for continuous analysis. Re-runs analysis when files change. (Experimental)                                                                                 |
| `--list-codes`  | List all available analyzer issue codes in JSON format.                                                                                                                       |
| `--help`, `-h`  | Print the help summary for the command.                                                                                                                                       |

### Reading from stdin (editor integration)

When using an editor or IDE that can pipe unsaved buffer content, you can run the analyzer on that content while still using the real file path for baseline lookup and issue locations:

```sh
cat src/Example.php | mago analyze --stdin-input src/Example.php
```

You must pass **exactly one path**; it is used as the logical file name (workspace-relative) for baseline matching and diagnostics. The path is normalized (e.g. `./src/Example.php` is treated like `src/Example.php`). This mode conflicts with `--staged` and `--watch`.

### Shared Reporting and Fixing Options

The `analyze` command shares a common set of options with other Mago tools for reporting, fixing, and baseline management.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)
