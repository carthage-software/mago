---
title: Linter usage
---

# Usage

The `mago lint` command is the entry point for running the linter.

## Linting your project

To lint all the source files defined in your `mago.toml` configuration, simply run:

```sh
mago lint
```

Mago will scan your project in parallel and report any issues it finds.

## Auto-fixing issues

Many lint rules provide automatic fixes. To apply them, use the `--fix` flag:

```sh
mago lint --fix
```

This will modify your files in place. To see what changes would be made without applying them, you can combine it with `--dry-run`:

```sh
mago lint --fix --dry-run
```

## Running specific rules

If you want to run only a specific set of rules, use the `--only` flag. This is great for incrementally introducing new rules to a project.

```sh
# Run only these two rules
mago lint --only no-empty,use-compound-assignment
```

For more details on the available command-line options, see the [Command Reference](./command-reference.md).
