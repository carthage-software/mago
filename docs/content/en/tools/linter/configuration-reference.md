+++
title = "Linter configuration reference"
description = "Every option Mago accepts under [linter] and [linter.rules]."
nav_order = 60
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Configuration reference

The linter is configured under two tables in `mago.toml`: `[linter]` for tool-wide settings and `[linter.rules]` for per-rule settings.

```toml
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

# Exclude specific paths from one rule
prefer-static-closure = { exclude = ["tests/"] }
```

## `[linter]`

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `excludes` | string list | `[]` | Paths or globs the linter skips. Additive to the global `source.excludes`. |
| `integrations` | string list | `[]` | Framework integrations to enable. The full list is on the [integrations page](/tools/linter/integrations/). |
| `baseline` | string | none | Path to a baseline file. When set, the linter uses it as the default baseline so you do not have to pass `--baseline` every run. CLI `--baseline` overrides this. |
| `baseline-variant` | string | `"loose"` | Variant for newly-generated baselines. Either `"loose"` (count-based, resilient) or `"strict"` (line-exact). See [baseline variants](/fundamentals/baseline/#two-variants). |
| `minimum-fail-level` | string | `"error"` | Lowest severity that triggers a non-zero exit. Values: `"note"`, `"help"`, `"warning"`, `"error"`. The CLI `--minimum-fail-level` overrides this. |

`excludes` here adds to the global list. Files matched globally are always excluded; this option lets you exclude additional files from the linter only.

```toml
[source]
excludes = ["cache/**"]              # excluded from every tool

[linter]
excludes = ["database/migrations/**"]  # additionally excluded from the linter only
```

## `[linter.rules]`

Each key under this table is a rule code, written in `kebab-case`. Every rule accepts the common options below; some rules also accept their own.

### Common options

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | boolean | varies | Enable or disable the rule. |
| `level` | string | varies | Severity. Values: `"error"`, `"warning"`, `"help"`, `"note"`. |
| `exclude` | string list | `[]` | Paths or globs the rule skips. Other rules still apply to those files. |

### Per-rule excludes

`exclude` is useful when a rule is generally valuable but not appropriate for one slice of the codebase, like generated code or test fixtures.

```toml
[linter.rules]
prefer-static-closure = { enabled = true, exclude = ["tests/"] }
no-goto              = { exclude = ["src/Legacy/"] }
no-eval              = { exclude = ["src/Templating/Compiler.php"] }
no-global            = { exclude = ["**/*Test.php"] }
```

Each entry can be a plain path or a glob:

- Plain paths (`"tests"`, `"tests/"`, `"src/Foo.php"`) match as prefixes against the relative file path from the project root.
- Glob patterns (any entry containing `*`, `?`, `[`, or `{`) match the full relative path using the same glob engine the global `source.excludes` uses, with the `[source.glob]` settings applied.

Glob patterns in per-rule `exclude` require Mago 1.20 or later. Earlier releases only accept plain prefix paths.

Per-rule `exclude` is not the same as `[linter].excludes`:

- `[linter].excludes` removes files from every rule.
- A rule's own `exclude` removes files from that one rule. Other rules still apply.

### Rule-specific options

Some rules accept additional options. `cyclomatic-complexity` is a typical example:

```toml
[linter.rules]
cyclomatic-complexity = { level = "error", threshold = 15 }
```

To discover the options for any specific rule, ask Mago:

```sh
mago lint --explain cyclomatic-complexity
```

The full per-rule reference is on the [rules page](/tools/linter/rules/).
