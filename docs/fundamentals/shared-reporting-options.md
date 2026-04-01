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
| `--reporting-format <FORMAT>`        | Choose the output format. See below for options. Default: auto-detected (see below).                                            |
| `--minimum-fail-level <LEVEL>`, `-m` | Set the minimum issue level that will cause a failure exit code. Options: `note`, `help`, `warning`, `error`. Defaults to the value set in the configuration file, or `error` if not configured. |
| `--minimum-report-level <LEVEL>`     | Set the minimum issue severity to include in the report. Issues below this level are filtered out.                              |
| `--retain-code <CODE>`               | Retain only issues with the specified code(s). Can be specified multiple times. **This is a reporting filter only** - all rules/checks still run. See below for details. |

:::info Difference between `--only` and `--retain-code`
The `--retain-code` option is **not the same** as the `--only` option available in `mago lint`:

- **`mago lint --only <RULE>`**: Runs **only** the specified rule(s). Other rules are completely skipped, improving performance.
- **`mago lint --retain-code <CODE>`**: Runs **all** rules, but filters the output to show only issues with the specified code(s).

**Example:**
```sh
# Run only the 'no-unused-variable' rule
mago lint --only no-unused-variable

# Run all rules, but show only 'no-unused-variable' issues in output
mago lint --retain-code no-unused-variable

# Retain multiple issue codes (runs all rules, shows only these codes)
mago lint --retain-code no-unused-variable --retain-code semantics

# For analyze command (no --only option available)
mago analyze --retain-code invalid-argument --retain-code type-mismatch
```

Use `--only` when you want faster execution by running fewer rules. Use `--retain-code` when you want to focus on specific issue types in the output while still getting the benefit of all checks running.
:::

### Reporting Formats

You can choose from several reporting formats with the `--reporting-format` flag:

- **Human-Readable:** `rich`, `medium`, `short`, `ariadne`, `emacs`
- **CI/CD & Machine-Readable:** `github`, `gitlab`, `json`, `checkstyle`
- **Summaries:** `count`, `code-count`

### Auto-Detection

When `--reporting-format` is not explicitly specified, Mago automatically selects the best format for your environment:

| Environment          | Detected via                | Default Format |
| :------------------- | :-------------------------- | :------------- |
| GitHub Actions       | `GITHUB_ACTIONS` env var    | `github`       |
| GitLab CI            | `GITLAB_CI` env var         | `gitlab`       |
| AI coding agents     | `CLAUDECODE`, `GEMINI_CLI`, `CODEX_SANDBOX`, or `OPENCODE_CLIENT` env vars | `medium` |
| Everything else      |                             | `rich`         |

This means CI/CD pipelines get native annotations and AI agents get a token-efficient format out of the box, with no configuration needed. You can always override with an explicit `--reporting-format` flag.

:::warning
Auto-detection is available starting from Mago 1.18.0. If you are using Mago 1.17.0 or earlier, you must explicitly pass `--reporting-format=github` or `--reporting-format=gitlab` for CI/CD annotations.
:::

## Baseline

These flags are used to manage baseline files for ignoring pre-existing issues. This feature is available for `mago lint` and `mago analyze`.

For a complete guide, see the [Baseline documentation](/fundamentals/baseline.md).

| Flag                  | Description                                                                                     |
| :-------------------- | :---------------------------------------------------------------------------------------------- |
| `--generate-baseline` | Generate a new baseline file, capturing all current issues.                                     |
| `--baseline <PATH>`   | Specify the path to a baseline file to use for ignoring issues.                                 |
| `--backup-baseline`   | Create a backup of the old baseline file (e.g., `baseline.toml.bkp`) when generating a new one. |
| `--ignore-baseline`   | Ignore any configured or specified baseline, reporting all issues.                               |
