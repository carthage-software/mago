---
title: Formatter usage
---

# Using the formatter

**Mago**'s formatter is designed to be simple to run. You can format your entire project, specific directories, or even code piped from `stdin`.

## Formatting your project

To format all the source files defined in your `mago.toml` configuration, simply run:

```sh
mago fmt
```

This command will find all relevant files and overwrite them in place with the formatted version.

## Checking for formatting issues

In a continuous integration environment, you'll want to check if files are formatted correctly
without actually changing them. The `--check` flag is perfect for this.

```sh
mago fmt --check
```

This command will exit with a success code (`0`) if all files are correctly formatted,
and a failure code (`1`) if any files would be changed. No output is printed on success, making it ideal
for scripts.

## Previewing changes

If you want to see what changes the formatter would make without modifying any files,
use the `--dry-run` flag.

```bash
mago fmt --dry-run
```

This will print a diff of all proposed changes to your console.

## Formatting specific directories or files

You can also format specific files or directories by passing them as arguments:

```bash
# Format a specific file
mago fmt src/Service.php

# Format a specific directory
mago fmt src/
```

## Formatting from stdin

You can format code directly from standard input (stdin). This is useful for integrating with other tools
or scripts.

```bash
cat src/Service.php | mago fmt --stdin-input
```

This will read the code from `src/Service.php`, format it, and print the formatted code to standard
output.

For more details on the available command-line options, see the [Command Reference](/tools/formatter/command-reference.md).
