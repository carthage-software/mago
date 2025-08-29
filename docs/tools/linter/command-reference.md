---
title: Linter command reference
outline: deep
---

# Command reference

The `mago lint` command is the entry point for all linting-related tasks.

```sh
Usage: mago lint [OPTIONS] [PATH]...
```

## Arguments

### `[PATH]...`

Optional. A list of specific files or directories to lint. If you provide paths here, they will be used instead of the `paths` defined in your `mago.toml` configuration.

## Options

### `--list-rules`

List all enabled linter rules and their descriptions.

### `--json`

Used with `--list-rules` to output the rule information in a machine-readable JSON format. This is primarily intended for documentation generation.

### `--explain <RULE_CODE>`

Provide detailed documentation for a specific linter rule (e.g., `no-redundant-nullsafe`).

### `-o`, `--only <RULE_CODE>`

Run only a specific, comma-separated list of rules, overriding the configuration file.

### `--pedantic`

Enable all linter rules for the most exhaustive analysis possible. This overrides your configuration, ignores PHP version constraints, and enables rules that are disabled by default.

:::warning
This mode is extremely noisy, not recommended for general use.
:::

### `-s`, `--semantics`

Perform only the parsing and basic semantic checks without running any lint rules.

### Auto-fixing

| Flag | Description |
| :--- | :--- |
| `--fix` | Automatically apply any safe fixes for the issues that are found. |
| `--fixable-only` | Filter the output to show only issues that have an automatic fix available. |
| `--unsafe` | Apply fixes that are marked as "unsafe" and may require manual verification. |
| `--potentially-unsafe` | Apply fixes that are marked as "potentially unsafe". |
| `--format-after-fix` | Automatically run the formatter on any files modified by `--fix`. |
| `-d`, `--dry-run` | Preview fixes as a diff without writing any changes to disk. |

### Reporting

| Flag | Description |
| :--- | :--- |
| `--sort` | Sort reported issues by level, code, and location. |
| `--reporting-format <FORMAT>` | Choose the output format (e.g., `rich`, `json`, `checkstyle`). Default: `rich`. |
| `-m`, `--minimum-fail-level <LEVEL>` | Set the minimum issue level (`note`, `help`, `warning`, `error`) that will cause a failure exit code. Default: `error`. |

### Baseline

| Flag | Description |
| :--- | :--- |
| `--generate-baseline` | Generate a baseline file (`mago-baseline.php`) to ignore all currently existing issues. |
| `--baseline <PATH>` | Specify a custom path to a baseline file to use for ignoring issues. |
| `--backup-baseline` | Backup the old baseline file when generating a new one. |

### `-h`, `--help`

Print the help summary for the command.
