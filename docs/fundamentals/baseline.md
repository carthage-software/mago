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

## How it Works

For each issue, Mago generates a unique signature based on the issue's code, message, and location (a hash of the file path and line number). This allows the baseline to be resilient to code changes that do not affect the issue itself.

## Maintaining the Baseline

Over time, as you fix issues that are part of the baseline, their entries in the baseline file become "dead" or "stale." Mago will detect this and warn you that your baseline file contains entries for issues that no longer exist.

When you see this warning, it is a good practice to regenerate the baseline file to keep it up-to-date:

```bash
mago lint --generate-baseline --baseline lint-baseline.toml
```

You can also use the `--backup-baseline` flag to create a backup of the old baseline file (e.g., `lint-baseline.toml.bkp`) before generating a new one.
