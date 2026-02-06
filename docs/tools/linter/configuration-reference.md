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

# Exclude specific paths from a rule
prefer-static-closure = { exclude = ["tests/"] }
```

## `[linter]`

| Option             | Type       | Default   | Description                                                                  |
| :----------------- | :--------- | :-------- | :--------------------------------------------------------------------------- |
| `excludes`         | `string[]` | `[]`      | A list of paths or glob patterns to exclude from linting.                    |
| `integrations`     | `string[]` | `[]`      | A list of framework integrations to enable (e.g., `"symfony"`, `"laravel"`). |
| `baseline`         | `string`   | `null`    | Path to a baseline file to ignore listed issues. When specified, the linter will use this file as the default baseline, eliminating the need to pass `--baseline` on every run. Command-line `--baseline` arguments will override this setting. |
| `baseline-variant` | `string`   | `"loose"` | The baseline format variant to use when generating new baselines. Options: `"loose"` (count-based, resilient to line changes) or `"strict"` (exact line matching). See [Baseline Variants](/fundamentals/baseline#baseline-variants) for details. |

:::tip Tool-Specific Excludes
The `excludes` option here is **additive** to the global `source.excludes` defined in the `[source]` section of your configuration. Files excluded globally will always be excluded from linting, and this option allows you to exclude additional files from the linter specifically.

For example:
```toml
[source]
excludes = ["cache/**"]  # Excluded from ALL tools

[linter]
excludes = ["database/migrations/**"]  # Additionally excluded from linter only
```
:::

## `[linter.rules]`

This table allows you to configure individual lint rules. Each key is the rule's code (in `kebab-case`).

### Common rule options

All rules accept these common options:

| Option    | Type       | Default | Description                                                                 |
| :-------- | :--------- | :------ | :-------------------------------------------------------------------------- |
| `enabled` | `boolean`  | (varies) | Enable or disable the rule.                                                |
| `level`   | `string`   | (varies) | Set the issue severity. Options: `"error"`, `"warning"`, `"help"`, `"note"`. |
| `exclude` | `string[]` | `[]`    | A list of paths to exclude from this specific rule.                         |

#### Per-rule path exclusions

The `exclude` option allows you to skip a rule for specific files or directories. This is useful when a rule is generally valuable but not appropriate for certain parts of your codebase, such as test files or generated code.

```toml
[linter.rules]
# Don't enforce static closures in test files
prefer-static-closure = { enabled = true, exclude = ["tests/"] }

# Disable goto checks only in legacy code
no-goto = { exclude = ["src/Legacy/"] }

# Exclude a specific file
no-eval = { exclude = ["src/Templating/Compiler.php"] }
```

Paths are matched as prefixes against relative file paths from the project root. Both `"tests"` and `"tests/"` will exclude all files under the `tests` directory.

:::tip
Per-rule `exclude` is different from the top-level `[linter].excludes`:
- `[linter].excludes` removes files from **all** lint rules.
- A rule's `exclude` removes files from **that specific rule** only — other rules still apply to those files.
:::

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
