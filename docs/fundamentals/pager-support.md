---
title: Pager Support
---

# Pager Support

:::info
This feature is only available from Mago version 1.0.0-beta.13 onwards.
:::

:::warning
The pager feature is not available on Windows systems.
:::

Mago integrates a pager to provide a better viewing experience for long output in your terminal.

## Pager-Enabled Commands

The pager is automatically activated for the following commands when their output is directed to `stdout` and is likely to exceed the screen height:

- `mago lint`
- `mago analyze`

The pager will **not** be used if you redirect the output to `stderr` using the `--reporting-target stderr` flag.

## Supported Formats

Paging is only enabled for reporting formats that are designed for human consumption in a terminal.

### Paging Supported

- `rich` (default)
- `medium`
- `short`
- `ariadne`
- `emacs`

### Paging Not Supported

These formats are intended for machine consumption or CI/CD pipelines and will always be printed directly to `stdout`.

- `github`
- `gitlab`
- `json`
- `count`
- `code-count`
- `checkstyle`

## Configuration

You can control the pager's behavior using [environment variables](/guide/environment-variables.md) or by passing the `--pager` flag to a supported command.

- `--pager`: Force the pager to be active for the current command.
- `--pager=false`: Disable the pager for the current command.

This flag will override the `use-pager` setting in your `mago.toml` file for a single run.
