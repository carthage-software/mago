+++
title = "Reporting and fixing options"
description = "The flags shared by lint, analyze, and ast for reporting issues, applying fixes, and managing baselines."
nav_order = 40
nav_section = "Fundamentals"
+++
# Reporting and fixing options

`mago lint`, `mago analyze`, and `mago ast` share a set of flags for how issues are reported and how fixes are applied. This page is the central reference for those flags so we don't repeat them on every command page.

## Auto-fixing

Most linter rules and a handful of analyzer checks ship automatic fixes. The flags below control how fixes are applied and which categories are eligible.

| Flag | Description |
| :--- | :--- |
| `--fix` | Apply every safe fix to issues found. |
| `--fixable-only`, `-f` | Filter the output to issues that have an automatic fix available. |
| `--unsafe` | Apply fixes marked unsafe. These may alter behaviour and need review. |
| `--potentially-unsafe` | Apply fixes marked potentially-unsafe. Less risky than unsafe but still worth a quick review. |
| `--format-after-fix`, `fmt` | Run the formatter on every file `--fix` modified. |
| `--dry-run`, `-d`, `diff` | Preview fixes as a unified diff without writing anything. |

## Reporting

How Mago presents the issues it finds.

| Flag | Description |
| :--- | :--- |
| `--sort` | Sort reported issues by level, then code, then location. |
| `--reporting-target <TARGET>` | Where to write the report. Values: `stdout` (default), `stderr`. |
| `--reporting-format <FORMAT>` | Output format. See below; defaults to auto-detected. |
| `--minimum-fail-level <LEVEL>`, `-m` | Lowest level that triggers a non-zero exit. Values: `note`, `help`, `warning`, `error`. Defaults to the value in the config file, or `error` if absent. |
| `--minimum-report-level <LEVEL>` | Lowest level included in the report. Issues below this are filtered out before printing. |
| `--retain-code <CODE>` | Keep only issues with the given code(s). Reporting filter, not an execution filter. Repeatable. |

`--retain-code` is not the same as `--only` (which only `mago lint` accepts):

- `mago lint --only <RULE>` runs only the specified rules. Other rules are skipped entirely, which is faster.
- `mago lint --retain-code <CODE>` runs every rule and filters the output to the codes you list.

```sh
mago lint --only no-unused-variable                                  # only run that rule
mago lint --retain-code no-unused-variable                           # run everything, show only this code
mago lint --retain-code no-unused-variable --retain-code semantics   # multiple codes
mago analyze --retain-code invalid-argument --retain-code type-mismatch
```

Use `--only` when you want a fast feedback loop on a specific rule. Use `--retain-code` when you want full coverage but a focused report.

### Reporting formats

Pick one explicitly with `--reporting-format`:

- Human-readable: `rich`, `medium`, `short`, `ariadne`, `emacs`.
- CI / machine-readable: `github`, `gitlab`, `json`, `checkstyle`, `sarif`.
- Summaries: `count`, `code-count`.

### Auto-detection

If `--reporting-format` is not set, Mago picks one based on the environment:

| Environment | Detected via | Default format |
| :--- | :--- | :--- |
| GitHub Actions | `GITHUB_ACTIONS` | `github` |
| GitLab CI | `GITLAB_CI` | `gitlab` |
| AI coding agents | `CLAUDECODE`, `GEMINI_CLI`, `CODEX_SANDBOX`, `OPENCODE_CLIENT` | `medium` |
| Anything else | (none) | `rich` |

CI pipelines therefore get native annotations and AI agents get a token-efficient format with no configuration. Pass `--reporting-format` explicitly to override.

> Auto-detection is available since Mago 1.18. On 1.17 and earlier, set `--reporting-format=github` or `--reporting-format=gitlab` explicitly.

## Baseline

Flags for managing baseline files. The full guide is on the [baseline page](/fundamentals/baseline/).

| Flag | Description |
| :--- | :--- |
| `--generate-baseline` | Generate a new baseline file capturing every current issue. |
| `--baseline <PATH>` | Use the baseline at the given path. |
| `--backup-baseline` | When regenerating, copy the old baseline to `<file>.bkp` before overwriting. |
| `--ignore-baseline` | Ignore any configured or specified baseline and report every issue. |
