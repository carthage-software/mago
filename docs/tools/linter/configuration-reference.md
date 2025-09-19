---
title: Linter configuration reference
---

# Configuration reference

**Mago**'s linter is configured in your `mago.toml` file under the `[linter]` and `[linter.rules]` tables.

```toml
# Example linter configuration
[linter]
integrations = ["symfony", "phpunit"]
excludes = ["src/Generated/"]
baseline = "linter-baseline.toml"

[linter.rules]
# Disable a rule completely
ambiguous-function-call = { enabled = false }

# Change a rule's severity level
no-else-clause = { level = "warning" }

# Configure a rule's specific options
cyclomatic-complexity = { threshold = 20 }
```

## `[linter]`

| Option         | Type       | Default | Description                                                                  |
| :------------- | :--------- | :------ | :--------------------------------------------------------------------------- |
| `excludes`     | `string[]` | `[]`    | A list of paths or glob patterns to exclude from linting.                    |
| `integrations` | `string[]` | `[]`    | A list of framework integrations to enable (e.g., `"symfony"`, `"laravel"`). |
| `baseline`     | `string`   | `null`  | Path to a baseline file to ignore listed issues. When specified, the linter will use this file as the default baseline, eliminating the need to pass `--baseline` on every run. Command-line `--baseline` arguments will override this setting. |

## `[linter.rules]`

This table allows you to configure individual lint rules. Each key is the rule's code (in `kebab-case`).

### Common rule options

All rules accept two common options:

- `enabled`: A boolean (`true` or `false`) to enable or disable the rule.
- `level`: A string to set the issue severity. Options are `"error"`, `"warning"`, `"help"`, and `"note"`.

### Rule-specific options

Some rules have additional configuration options. For example, `cyclomatic-complexity` has a `threshold`:

```toml
[linter.rules]
cyclomatic-complexity = { level = "error", threshold = 15 }
```

To find the specific options available for any rule, the best and most up-to-date method is to use the `--explain` command:

```sh
mago lint --explain cyclomatic-complexity
```
