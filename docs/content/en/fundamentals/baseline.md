+++
title = "Baseline"
description = "Snapshot existing issues so Mago only flags new ones, with two variants for different precision-vs-resilience trade-offs."
nav_order = 20
nav_section = "Fundamentals"
+++
# Baseline

A baseline file records the issues that exist in your codebase right now and tells Mago to ignore them on future runs. New issues introduced after the baseline still get flagged. Useful when adopting Mago in a project that already has hundreds or thousands of issues, or when staging a tightening of rules over multiple PRs.

## One file per tool

The linter and analyzer each carry their own baseline because the issues they report are different. Conventional names:

- Linter: `lint-baseline.toml`
- Analyzer: `analysis-baseline.toml`

The `mago ast` command reports parse errors and does not support a baseline.

## Generating a baseline

```sh
mago lint --generate-baseline --baseline lint-baseline.toml
mago analyze --generate-baseline --baseline analysis-baseline.toml
```

The command runs the tool, collects every issue it finds, and serialises them into the specified TOML file.

## Using a baseline

```sh
mago lint --baseline lint-baseline.toml
mago analyze --baseline analysis-baseline.toml
```

When a baseline is in use, Mago:

1. Finds every issue in the current code.
2. Compares them against the baseline.
3. Suppresses matches.
4. Reports only what is left.

You can also set the baseline path in `mago.toml` so you don't have to pass `--baseline` every time:

```toml
[linter]
baseline = "lint-baseline.toml"

[analyzer]
baseline = "analysis-baseline.toml"
```

## Two variants

Mago supports two baseline shapes with different precision-vs-resilience trade-offs.

### Loose (default)

Groups issues by `(file, code, message)` and stores a count. Resilient to line-number shifts: if you reformat or insert code above an issue, the baseline still matches as long as the same kind of issue still occurs.

```toml
variant = "loose"

[[issues]]
file    = "src/Service/PaymentProcessor.php"
code    = "possibly-null-argument"
message = "Argument #1 of `process` expects `Order`, but `?Order` was given."
count   = 2
```

### Strict

Stores exact line ranges per issue. Precise, but the baseline goes stale every time line numbers shift, so you regenerate often.

```toml
variant = "strict"

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 42
end_line   = 42

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 87
end_line   = 90
```

### When to pick which

| Variant | Best for | Trade-off |
| :--- | :--- | :--- |
| Loose | Most projects, CI pipelines | Resilient to refactoring, less precise. |
| Strict | When exact line tracking matters | Precise, but requires frequent regeneration. |

Set the variant for new baseline files in `mago.toml`:

```toml
[linter]
baseline = "lint-baseline.toml"
baseline-variant = "loose"   # or "strict"

[analyzer]
baseline = "analysis-baseline.toml"
baseline-variant = "loose"
```

The setting only affects generation. When reading an existing baseline, Mago detects the variant from the file's `variant` header.

### Backwards compatibility

Baseline files written by older Mago releases (before variant support) have no `variant` header. Mago treats those as strict and prints a warning recommending you regenerate the file so it gains the header.

## Skipping the baseline temporarily

```sh
mago lint --ignore-baseline
mago analyze --ignore-baseline
```

Useful when you want to see issues currently suppressed by the baseline, for example to clean some of them up.

## Keeping the baseline tidy

When you fix an issue that was part of the baseline, its entry becomes dead. Mago detects this and warns about stale entries. Regenerate to clean up:

```sh
mago lint --generate-baseline --baseline lint-baseline.toml
```

Pass `--backup-baseline` to keep the previous file as `lint-baseline.toml.bkp` before overwriting.

## JSON Schema

If you build tooling or IDE integration that needs to parse or generate baseline files, get the schema:

```sh
mago config --schema --show baseline
```

The output is a JSON Schema (draft 2020-12) covering both variants.
