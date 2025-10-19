---
title: Analyzer configuration Reference
---

# Configuration reference

**Mago**'s analyzer is highly configurable, allowing you to tailor the analysis to your project's specific needs. All settings go under the `[analyzer]` table in your `mago.toml` file.

```toml
[analyzer]
# Disable a specific issue category
redundancy-issues = false

# Ignore a specific error code across the whole project
ignore = ["mixed-argument"]

# Use a baseline file to ignore existing issues
baseline = "analyzer-baseline.toml"
```

## General options

| Option     | Type       | Default | Description                                                |
| :--------- | :--------- | :------ | :--------------------------------------------------------- |
| `excludes` | `string[]` | `[]`    | A list of paths or glob patterns to exclude from analysis. |
| `ignore`   | `string[]` | `[]`    | A list of specific issue codes to ignore globally.         |
| `baseline` | `string`   | `null`  | Path to a baseline file to ignore listed issues. When specified, the analyzer will use this file as the default baseline, eliminating the need to pass `--baseline` on every run. Command-line `--baseline` arguments will override this setting. |

## Issue categories

You can enable or disable entire categories of issues. All categories are enabled by default.

| Option                 | Default | Description                                                   |
| :--------------------- | :------ | :------------------------------------------------------------ |
| `mixed-issues`         | `true`  | Report all issues related to the use of `mixed` types.        |
| `falsable-issues`      | `true`  | Report all issues related to possibly `false` values.         |
| `nullable-issues`      | `true`  | Report all issues related to possibly `null` values.          |
| `redundancy-issues`    | `true`  | Report all issues related to redundant code.                  |
| `reference-issues`     | `true`  | Report all issues related to by-reference variables.          |
| `unreachable-issues`   | `true`  | Report all issues related to unreachable code.                |
| `deprecation-issues`   | `true`  | Report all issues related to using deprecated code.           |
| `impossibility-issues` | `true`  | Report all issues related to logically impossible conditions. |
| `ambiguity-issues`     | `true`  | Report all issues related to ambiguous code constructs.       |
| `existence-issues`     | `true`  | Report all issues related to the existence of symbols.        |
| `template-issues`      | `true`  | Report all issues related to generic template types.          |
| `argument-issues`      | `true`  | Report all issues related to function arguments.              |
| `operand-issues`       | `true`  | Report all issues related to expression operands.             |
| `property-issues`      | `true`  | Report all issues related to class properties.                |
| `generator-issues`     | `true`  | Report all issues related to generators.                      |
| `array-issues`         | `true`  | Report all issues related to array operations.                |
| `return-issues`        | `true`  | Report issues related to function and method return types.    |
| `method-issues`        | `true`  | Report issues related to methods and their usage.             |
| `iterator-issues`      | `true`  | Report issues related to iterators and their usage.           |

## Feature flags

These flags control specific, powerful analysis capabilities.

| Option                                | Default | Description                                                                                          |
| :------------------------------------ | :------ | :--------------------------------------------------------------------------------------------------- |
| `find-unused-expressions`             | `true`  | Find and report expressions whose results are not used (e.g., `$a + $b;`).                           |
| `find-unused-definitions`             | `true`  | Find and report unused definitions (e.g., private methods that are never called).                    |
| `analyze-dead-code`                   | `true`  | Analyze code that appears to be unreachable.                                                         |
| `memoize-properties`                  | `false` | Track the literal values of class properties. Improves type inference but may increase memory usage. |
| `allow-possibly-undefined-array-keys` | `true`  | Allow accessing array keys that may not be defined without reporting an issue.                       |
| `check-throws`                        | `true`  | Check for unhandled thrown exceptions that are not caught or documented with `@throws`.              |
| `perform-heuristic-checks`            | `true`  | Perform extra heuristic checks for potential issues that are not strict typing errors.               |
| `strict-list-index-checks`            | `false` | When `true`, requires any integer used as a `list` index to be provably non-negative.                |
| `no-boolean-literal-comparison`       | `false` | When `true`, disallows direct comparison to boolean literals (e.g., `$a === true`).                  |
| `check-missing-type-hints`            | `false` | When `true`, reports missing type hints on parameters, properties, and return types.                 |
| `check-closure-missing-type-hints`    | `false` | When `true`, checks closures for missing type hints when `check-missing-type-hints` is enabled.      |
| `check-arrow-function-missing-type-hints` | `false` | When `true`, checks arrow functions for missing type hints when `check-missing-type-hints` is enabled. |
| `register-super-globals`              | `true`  | Automatically register PHP superglobals (e.g., `$_GET`, `$_POST`) for analysis.                      |
