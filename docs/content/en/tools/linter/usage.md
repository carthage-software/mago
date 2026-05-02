+++
title = "Linter usage"
description = "Common ways to drive mago lint, including auto-fixing and running a single rule."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Usage

The entry point is `mago lint`. It runs the linter against the source files declared in `mago.toml` (or against arguments you pass on the command line).

## Lint the whole project

```sh
mago lint
```

Mago scans the project in parallel and reports every issue it finds.

## Apply automatic fixes

Most rules ship a safe fix. To rewrite the affected files in place:

```sh
mago lint --fix
```

To preview the fixes as a unified diff without touching disk:

```sh
mago lint --fix --dry-run
```

To run the formatter on every file the fixer rewrote, append `--format-after-fix`:

```sh
mago lint --fix --format-after-fix
```

Less-safe fixes are gated. Use `--potentially-unsafe` to opt into fixes that may need a quick review, and `--unsafe` for the ones that may alter behaviour. Combined with `--dry-run` you can see exactly what would change before committing.

## Run a single rule (or a few)

`--only` runs only the listed rules and skips the rest. Faster than running the full catalogue, useful for incremental adoption.

```sh
mago lint --only no-empty
mago lint --only no-empty,use-compound-assignment
```

If you want every rule to run but only see issues for a subset of codes, use `--retain-code` instead. See the [reporting and fixing options](/fundamentals/shared-reporting-options/) for the full list of report-control flags.

## Lint specific files

Pass paths after the subcommand to limit the run to just those files or directories. Useful in pre-commit hooks against staged changes.

```sh
mago lint src/Service/PaymentProcessor.php
mago lint src/Service tests/Unit
```

The full list of flags is on the [command reference](/tools/linter/command-reference/).
