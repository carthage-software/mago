---
title: Formatter command reference
outline: deep
---

# Command reference

The `mago format` command is the entry point for all formatting-related tasks.

```sh
Usage: mago format [OPTIONS] [PATH]...
```

:::info
`mago fmt` is a convenient alias for `mago format`. Both can be used interchangeably.
:::

## Arguments

#### `[PATH]...`

Optional. A list of specific files or directories to format. If you provide paths here, they will be used instead
of the `paths` defined in your `mago.toml` configuration.

```bash
# Format a single file and a directory
mago fmt src/index.php tests/
```

## Options

### `-d`, `--dry-run`

Perform a "dry run". This will calculate and print a diff of all changes that would be made to your files without actually modifying them.

### `-c`, `--check`

Check if the source files are formatted correctly. This is the ideal flag for CI environments. The command will:

- Exit with code `0` if all files are formatted.
- Exit with code `1` if any files need formatting.

It does not modify any files or print any output to `stdout` on success.

### `-i`, `--stdin-input`

Read source code from `stdin`, format it, and print the result to `stdout`. This is useful for editor integrations.

```bash
echo "<?php\necho 'hello world';" | mago fmt --stdin-input
```

### `-h`, `--help`

Display help information about the `mago format` command.
