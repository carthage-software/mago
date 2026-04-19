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

| Flag, Alias(es)              | Description                                                                                                                                                                   |
| :--------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--no-stubs`                 | Analyze the project without loading the built-in PHP stubs for the standard library.                                                                                          |
| `--staged`                   | Only analyze files that are staged in git. Designed for pre-commit hooks. Fails if not in a git repository.                                                                   |
| `--stdin-input`              | Read file content from stdin and use the single path argument for baseline and reporting. Intended for editor integrations (e.g. unsaved buffers). Requires exactly one path. |
| `--substitute <ORIG=TEMP>`   | Replace a host file with another file for this invocation. Intended for mutation-testing frameworks. Can be repeated. See [Substituting files](#substituting-files) below.    |
| `--watch`                    | Enable watch mode for continuous analysis. Re-runs analysis when files change. See [Watch Mode](#watch-mode) below.                                                           |
| `--list-codes`               | List all available analyzer issue codes in JSON format.                                                                                                                       |
| `--help`, `-h`               | Print the help summary for the command.                                                                                                                                       |

### Reading from stdin (editor integration)

When using an editor or IDE that can pipe unsaved buffer content, you can run the analyzer on that content while still using the real file path for baseline lookup and issue locations:

```sh
cat src/Example.php | mago analyze --stdin-input src/Example.php
```

You must pass **exactly one path**; it is used as the logical file name (workspace-relative) for baseline matching and diagnostics. The path is normalized (e.g. `./src/Example.php` is treated like `src/Example.php`). This mode conflicts with `--staged` and `--watch`.

### Substituting files

`--substitute ORIG=TEMP` replaces one host file in the project with another file for the duration of a single invocation, without modifying anything on disk. It is primarily designed for mutation-testing frameworks (such as Infection) that generate a mutated copy of a source file and want the analyzer to evaluate the mutation against the rest of the project. If the analyzer already reports a new error on the mutated file, the mutation can be marked as killed without running the unit tests.

```sh
mago analyze --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Rules:

- Both `ORIG` and `TEMP` must be absolute paths and both files must exist.
- `ORIG` must be a host file in the project (under one of your configured `paths`). Vendored or excluded files cannot be substituted.
- The flag can be given multiple times in a single invocation to substitute several files at once.
- Conflicts with `--stdin-input` and `--staged`.

Under the hood, `TEMP` is added to the host paths and `ORIG` is added to the excludes for this run. The rest of the project is scanned as usual, so cross-file type inference continues to see the mutation. Reported issues and baseline entries reference the `TEMP` path, not `ORIG`; mutation-testing tools typically parse the diff of issue counts between a clean run and the substituted run, so this does not affect the workflow.

### Shared Reporting and Fixing Options

The `analyze` command shares a common set of options with other Mago tools for reporting, fixing, and baseline management.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)

## Watch Mode

When `--watch` is enabled, the analyzer continuously monitors your workspace for changes and automatically re-runs analysis whenever PHP files are modified, created, or deleted.

```sh
mago analyze --watch
```

### Automatic Restart

In addition to PHP file changes, the analyzer watches for changes to project configuration files. When any of the following files change, the analyzer **automatically restarts** with the reloaded configuration:

- **Configuration file** — the `mago.toml` (or whichever config file was loaded)
- **Baseline file** — if configured via `analyzer.baseline`
- **`composer.json`** and **`composer.lock`**

This means you can edit your `mago.toml` (e.g., add an ignore rule or change a setting) and the analyzer will pick up the changes without needing to manually restart.

:::tip
If no configuration file exists when watch mode starts, the analyzer watches for the creation of any supported config file (`mago.toml`, `mago.yaml`, `mago.json`, etc.) and restarts when one appears.
:::

Press **Ctrl+C** to stop watching.
