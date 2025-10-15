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

The `guard` command currently does not have any specific flags of its own, but it shares the common set of options for reporting issues.

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
