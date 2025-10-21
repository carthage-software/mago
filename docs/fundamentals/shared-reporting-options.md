---
title: Shared Reporting and Fixing Options
---

# Shared Reporting and Fixing Options

The `mago lint`, `mago analyze`, and `mago ast` commands share a common set of options for reporting issues, applying fixes, and managing baselines.

## Auto-Fixing

These options control how Mago automatically corrects issues.

| Flag, Alias(es)             | Description                                                                                                                                                |
| :-------------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--fix`                     | Automatically apply any safe fixes for the issues that are found.                                                                                          |
| `--fixable-only`, `-f`      | Filter the output to show only issues that have an automatic fix available.                                                                                |
| `--unsafe`                  | Apply fixes that are marked as "unsafe." Unsafe fixes might have unintended consequences or alter the code's behavior and may require manual verification. |
| `--potentially-unsafe`      | Apply fixes that are marked as "potentially unsafe." These are less risky than unsafe fixes but may still require manual review.                           |
| `--format-after-fix`, `fmt` | Automatically run the formatter on any files that have been modified by `--fix`.                                                                           |
| `--dry-run`, `-d`, `diff`   | Preview fixes as a diff without writing any changes to disk.                                                                                               |

## Reporting

These options customize how Mago reports the issues it finds.

| Flag, Alias(es)                      | Description                                                                                                                     |
| :----------------------------------- | :------------------------------------------------------------------------------------------------------------------------------ |
| `--sort`                             | Sort reported issues by level, code, and location.                                                                              |
| `--reporting-target <TARGET>`        | Specify where to report results. Options: `stdout`, `stderr`. Default: `stdout`.                                                |
| `--reporting-format <FORMAT>`        | Choose the output format. See below for options. Default: `rich`.                                                               |
| `--minimum-fail-level <LEVEL>`, `-m` | Set the minimum issue level that will cause a failure exit code. Options: `note`, `help`, `warning`, `error`. Default: `error`. |

### Reporting Formats

You can choose from several reporting formats with the `--reporting-format` flag:

- **Human-Readable:** `rich`, `medium`, `short`, `ariadne`, `emacs`
- **CI/CD & Machine-Readable:** `github`, `gitlab`, `json`, `checkstyle`
- **Summaries:** `count`, `code-count`

## Baseline

These flags are used to manage baseline files for ignoring pre-existing issues. This feature is available for `mago lint` and `mago analyze`.

For a complete guide, see the [Baseline documentation](/fundamentals/baseline.md).

| Flag                  | Description                                                                                     |
| :-------------------- | :---------------------------------------------------------------------------------------------- |
| `--generate-baseline` | Generate a new baseline file, capturing all current issues.                                     |
| `--baseline <PATH>`   | Specify the path to a baseline file to use for ignoring issues.                                 |
| `--backup-baseline`   | Create a backup of the old baseline file (e.g., `baseline.toml.bkp`) when generating a new one. |
