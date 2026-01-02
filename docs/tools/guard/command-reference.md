---
title: Guard Command Reference
outline: deep
---

# Command Reference

The `mago guard` command is the entry point for running Mago's architectural guard.

:::tip
For global options that can be used with any command, see the [Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to specify global options **before** the `guard` command.
:::

```sh
Usage: mago guard [OPTIONS] [PATHS]...
```

## Arguments

### `[PATHS]...`

Optional. A list of specific files or directories to analyze. If you provide paths here, they will be used instead of the `paths` defined in your `mago.toml` configuration.

## Options

### Mode Selection

These flags control which guard checks are executed. They are mutually exclusive.

| Flag           | Description                                                                                     |
| :------------- | :---------------------------------------------------------------------------------------------- |
| `--structural` | Run only structural guard checks (naming conventions, modifiers, inheritance constraints).       |
| `--perimeter`  | Run only perimeter guard checks (dependency boundaries, layer restrictions).                     |

If neither flag is specified, both structural and perimeter guards will run (equivalent to `mode = "default"` in configuration).

:::tip
These flags override the `mode` setting in your `mago.toml` configuration. If you specify a flag that matches the configured mode, a warning will be shown indicating the flag is redundant.
:::

### Other Options

| Flag         | Description                                                                                      |
| :----------- | :----------------------------------------------------------------------------------------------- |
| `--no-stubs` | Disable built-in PHP and library stubs. May result in more warnings when external symbols can't be resolved. |

### Shared Reporting Options

The `guard` command uses a shared set of options for reporting the issues it finds.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)

:::info
Auto-fixing and baseline features are not applicable to the `guard` command.
:::

### Help

| Flag, Alias(es) | Description                             |
| :-------------- | :-------------------------------------- |
| `--help`, `-h`  | Print the help summary for the command. |
