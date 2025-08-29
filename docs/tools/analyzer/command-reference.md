---
title: Analyzer command reference
outline: deep
---

# Command reference

The `mago analyze` command is the entry point for running Mago's static type checker.

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

### `--no-stubs`

Analyze the project without loading the built-in PHP stubs for the standard library. Disabling stubs may lead to a large number of "symbol not found" errors if your code relies on standard PHP features.

### `--fix`

Automatically apply any safe fixes for the issues that are found.

### `--fixable-only`

Filter the output to show only issues that have an automatic fix available.

### `--unsafe`

Apply fixes that are marked as "unsafe". Unsafe fixes might have unintended consequences or alter the code's behavior in a way that requires manual verification.

### `--potentially-unsafe`

Apply fixes that are marked as "potentially unsafe". These are less risky than unsafe fixes but may still require manual review.

### `--format-after-fix`

Automatically run the formatter on any files that have been modified by the `--fix` command.

### `-d`, `--dry-run`

Preview fixes without writing any changes to disk. This option shows a diff of what changes would be made if fixes were applied.

### `--generate-baseline`

Generate a baseline file (`mago-baseline.php`) to ignore all currently existing issues. This is useful for introducing Mago to a legacy codebase.

### `--baseline <PATH>`

Specify a custom path to a baseline file to use for ignoring issues.

### `--reporting-format <FORMAT>`

Choose the format for the output.

- **Default:** `rich`
- **Options:** `rich`, `medium`, `short`, `ariadne`, `github`, `gitlab`, `json`, `count`, `code-count`, `checkstyle`, `emacs`

### `-m`, `--minimum-fail-level <LEVEL>`

Set the minimum issue level that will cause the command to exit with a failure code. For example, if set to `error`, warnings or notices will not cause a failure.

- **Default:** `error`
- **Options:** `note`, `help`, `warning`, `error`

### `-h`, `--help`

Print the help summary for the command.
