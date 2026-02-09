---
title: Baseline
---

# Baseline

The baseline feature in Mago allows you to establish a snapshot of existing issues in your codebase. When you run Mago with a baseline file, it will ignore any issues that are already documented in that file, allowing you to focus only on new issues introduced in your changes.

This is particularly useful when integrating Mago into a large, existing project that may have hundreds or thousands of issues. Instead of being overwhelmed by a massive initial report, you can generate a baseline to acknowledge the current state and then work on preventing new issues from being added.

## Per-Tool Baselines

The `mago lint` and `mago analyze` commands each use their own separate baseline file. This is because the issues they report are different, and you may want to manage their baselines independently.

It is conventional to name the baseline files accordingly:

- For the linter: `lint-baseline.toml`
- For the analyzer: `analysis-baseline.toml`

:::tip
The `mago ast` command reports parsing errors but does not support using a baseline to ignore them.
:::

## Generating a Baseline

You can generate a baseline file by running `lint` or `analyze` with the `--generate-baseline` flag. You must also specify the path where the baseline file will be stored using the `--baseline` flag.

```bash
# Generate a baseline for the linter
mago lint --generate-baseline --baseline lint-baseline.toml

# Generate a baseline for the analyzer
mago analyze --generate-baseline --baseline analysis-baseline.toml
```

This will execute the command, collect all found issues, and serialize them into the specified TOML file.

## Using a Baseline

Once you have a baseline file, instruct Mago to use it by passing the `--baseline` flag:

```bash
# Run the linter using its baseline
mago lint --baseline lint-baseline.toml

# Run the analyzer using its baseline
mago analyze --baseline analysis-baseline.toml
```

When you run a command with a baseline, Mago will:

1. Find all issues in the current code.
2. Compare the found issues against the ones listed in the specified baseline file.
3. Filter out any issues that match the baseline, so they are not reported.
4. Display only the new, unreported issues.

## Baseline Variants

Mago supports two baseline variants, each with different trade-offs between precision and resilience to code changes:

### Loose Variant (Default)

The **loose** variant groups issues by `(file, code, message)` and stores a count of occurrences. This makes it resilient to line number changes - if you add or remove lines in a file, the baseline remains valid as long as the same issues exist.

```toml
variant = "loose"

[[issues]]
file = "src/Service/PaymentProcessor.php"
code = "possibly-null-argument"
message = "Argument #1 of `process` expects `Order`, but `?Order` was given."
count = 2
```

### Strict Variant

The **strict** variant stores exact line numbers for each issue. This provides precise tracking but requires regenerating the baseline when line numbers shift due to code changes.

```toml
variant = "strict"

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 42
end_line = 42

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 87
end_line = 90
```

### Choosing a Variant

| Variant | Best For                         | Trade-off                                   |
| ------- | -------------------------------- | ------------------------------------------- |
| Loose   | Most projects, CI pipelines      | More resilient to refactoring, less precise |
| Strict  | When exact line tracking matters | Precise, but requires frequent regeneration |

### Configuring the Variant

You can configure which variant to use when generating new baselines in your `mago.toml`:

```toml
[linter]
baseline = "lint-baseline.toml"
baseline-variant = "loose"  # or "strict"

[analyzer]
baseline = "analysis-baseline.toml"
baseline-variant = "loose"  # or "strict"
```

The variant setting only affects **generation** of new baselines. When **reading** an existing baseline, Mago automatically detects the variant from the file's `variant` header.

### Backward Compatibility

Baseline files created with older versions of Mago (before variant support) don't have a `variant` header. When Mago encounters such a file, it assumes the **strict** variant and displays a warning recommending you regenerate the baseline to add the header.

## Ignoring the Baseline

If you want to temporarily see all issues — including those suppressed by the baseline — you can use the `--ignore-baseline` flag:

```bash
mago lint --ignore-baseline
mago analyze --ignore-baseline
```

This is useful when you want to fix issues that are currently in the baseline without having to edit or remove your baseline file.

## Maintaining the Baseline

Over time, as you fix issues that are part of the baseline, their entries in the baseline file become "dead" or "stale." Mago will detect this and warn you that your baseline file contains entries for issues that no longer exist.

When you see this warning, it is a good practice to regenerate the baseline file to keep it up-to-date:

```bash
mago lint --generate-baseline --baseline lint-baseline.toml
```

You can also use the `--backup-baseline` flag to create a backup of the old baseline file (e.g., `lint-baseline.toml.bkp`) before generating a new one.
