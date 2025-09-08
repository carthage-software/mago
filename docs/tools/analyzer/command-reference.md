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
| `--help`, `-h`  | Print the help summary for the command.                                              |

### Shared Reporting and Fixing Options

The `analyze` command shares a common set of options with other Mago tools for reporting, fixing, and baseline management.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)
