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

| Flag                       | Description                                                                                                                                                                   |
|:---------------------------|:------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--no-stubs`               | Disable built-in PHP and library stubs. May result in more warnings when external symbols can't be resolved.                                                                  |
| `--stdin-input`            | Read file content from stdin and use the single path argument for baseline and reporting. Intended for editor integrations (e.g. unsaved buffers). Requires exactly one path. |
| `--substitute <ORIG=TEMP>` | Replace a host file with another file for this invocation. Intended for mutation-testing frameworks. Can be repeated. See [Substituting files](#substituting-files) below.    |

### Reading from stdin (editor integration)

When using an editor or IDE that can pipe unsaved buffer content, you can run the guard on that content while still using the real file path for baseline lookup and issue locations:

```sh
cat src/Example.php | mago guard --stdin-input src/Example.php
```

You must pass **exactly one path**; it is used as the logical file name (workspace-relative) for baseline matching and diagnostics. The path is normalized (e.g. `./src/Example.php` is treated like `src/Example.php`).

### Substituting files

`--substitute ORIG=TEMP` replaces one host file in the project with another file for the duration of a single invocation, without modifying anything on disk. It is primarily designed for mutation-testing frameworks that generate a mutated copy of a source file and want the guard to evaluate the mutation against the rest of the project.

```sh
mago guard --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

Rules:

- Both `ORIG` and `TEMP` must be absolute paths and both files must exist.
- `ORIG` must be a host file in the project (under one of your configured `paths`). Vendored or excluded files cannot be substituted.
- The flag can be given multiple times in a single invocation to substitute several files at once.
- Conflicts with `--stdin-input`.

Under the hood, `TEMP` is added to the host paths and `ORIG` is added to the excludes for this run. The rest of the project is scanned as usual, so layer and namespace checks continue to see the mutation. Reported issues reference the `TEMP` path rather than `ORIG`; mutation-testing tools typically parse the diff of issue counts between a clean run and the substituted run, so this does not affect the workflow.

### Auto-Fix Options

The `guard` command can automatically fix structural violations for rules that support it (modifier constraints, interface/trait/extends additions). The `--fix` flag works the same way as in `mago lint`.

| Flag                   | Description                                                                                        |
|:-----------------------|:---------------------------------------------------------------------------------------------------|
| `--fix`                | Automatically apply fixes for fixable structural violations.                                       |
| `--dry-run`            | Preview what would be changed without writing any files. Shows a unified diff per file.            |
| `--format-after-fix`   | Run the formatter on every file changed by `--fix` to clean up whitespace and style.              |

```sh
# Preview fixes without writing files
mago guard --fix --dry-run

# Apply fixes automatically
mago guard --fix

# Apply fixes then format changed files
mago guard --fix --format-after-fix
```

:::tip
Not all structural violations can be auto-fixed. Flaws that require human judgment (renaming, changing class kind, adding abstract) are always reported but never auto-applied. See [Using the Guard — Auto-Fix](./usage.md#auto-fixing-structural-flaws) for the full list.
:::

### Shared Reporting Options

The `guard` command uses a shared set of options for reporting the issues it finds.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)

### Help

| Flag, Alias(es) | Description                             |
| :-------------- | :-------------------------------------- |
| `--help`, `-h`  | Print the help summary for the command. |
