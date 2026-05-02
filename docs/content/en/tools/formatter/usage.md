+++
title = "Formatter usage"
description = "Common ways to drive mago format: in place, dry-run, check mode, stdin, and pre-commit."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Usage

`mago format` (alias `mago fmt`) is the entry point. By default it formats every source file declared in `mago.toml` in place.

## Format the project

```sh
mago format
```

Files are rewritten in place. Run it after pulling, before committing, or as a step in CI.

## CI: check without rewriting

In a continuous integration step, you usually want to verify the project is already formatted without modifying anything. The `--check` flag does exactly that:

```sh
mago format --check
```

Exits `0` when every file is already formatted, `1` when at least one file would change. No output on success, so it stays quiet on the happy path.

## Preview changes

To see what the formatter would do without writing anything to disk, ask for a dry run:

```sh
mago format --dry-run
```

The output is a unified diff of the proposed changes.

## Specific files or directories

Pass paths after the subcommand to limit the run:

```sh
mago format src/Service.php
mago format src/ tests/
```

## Read from stdin

Useful when piping a buffer from an editor or another tool. Reads from stdin, prints the formatted result to stdout.

```sh
cat src/Service.php | mago format --stdin-input
```

Editor integrations should also pass the buffer's path so excludes can apply and parse-error messages name the real file:

```sh
cat src/Service.php | mago format --stdin-input --stdin-filepath src/Service.php
```

If the path matches an exclude pattern, the input is echoed back unchanged. Relative and absolute paths are both accepted.

## Pre-commit (staged files only)

`--staged` formats only the files currently staged in git, then re-stages them. Designed for pre-commit hooks where you want to avoid touching the working tree's unstaged changes.

```sh
mago format --staged
```

The [pre-commit recipe](/recipes/pre-commit-hooks/) walks through a complete hook setup.

The full flag list is on the [command reference](/tools/formatter/command-reference/).
