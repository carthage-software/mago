---
title: Formatter command reference
outline: deep
---

# Command reference

The `mago format` command is the entry point for all formatting-related tasks.

:::tip
For global options that can be used with any command, see the [Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to specify global options **before** the `format` command.
:::

```sh
Usage: mago format [OPTIONS] [PATH]...
```

:::info
`mago fmt` is a convenient alias for `mago format`. Both can be used interchangeably.
:::

## Arguments

#### `[PATH]...`

Optional. A list of specific files or directories to format. If you provide paths here, they will be used instead of the `paths` defined in your `mago.toml` configuration.

```bash
# Format a single file and a directory
mago fmt src/index.php tests/
```

## Options

| Flag, Alias(es)       | Description                                                                                                                                                                       |
| :-------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--dry-run`, `-d`     | Perform a "dry run". This will calculate and print a diff of all changes that would be made to your files without actually modifying them.                                        |
| `--check`, `-c`       | Check if the source files are formatted correctly. This is the ideal flag for CI environments. The command will exit with code `0` if all files are formatted, and `1` otherwise. |
| `--stdin-input`, `-i` | Read source code from `stdin`, format it, and print the result to `stdout`. This is useful for editor integrations.                                                               |
| `--help`, `-h`        | Display help information about the `mago format` command.                                                                                                                         |
