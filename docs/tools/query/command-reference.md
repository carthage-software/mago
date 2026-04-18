---
title: Query command reference
outline: deep
---

# Command reference

The `mago query` command runs a GritQL-style pattern across the codebase in parallel.
See [Query overview](./overview.md) for the pattern language and recipes.

::: warning Experimental
The query engine is experimental. Pattern syntax, output, and CLI flags may change
without warning between releases.
:::

::: tip
For global options that can be used with any command, see the
[Command-Line Interface overview](/fundamentals/command-line-interface.md). Remember to
specify global options **before** the `query` command.
:::

```sh
Usage: mago query [OPTIONS] <PATTERN> [PATH]...
```

## Arguments

### `<PATTERN>`

Required. The pattern to match. May be a bare PHP snippet (for simple searches) or a
full surface-grammar expression with backticked snippets (for rewrites, combinators,
and multi-part patterns). See the [overview](./overview.md) for complete syntax.

### `[PATH]...`

Optional. A list of files or directories to query. If provided, these paths override
the `paths` in your `mago.toml` `[source]` section. When omitted, the command runs
over every PHP file in the workspace.

## Shared Reporting and Fixing Options

`query` inherits the same reporting and fixing flags as `lint`, `guard`, and
`analyze`. Every match is emitted as a Help-level issue with code `query-match`, and
any rewrites produced by the pattern are attached as edits applied via `--fix`.

[**See the Shared Reporting and Fixing Options documentation.**](/fundamentals/shared-reporting-options.md)

## Examples

### Searching

```sh
mago query 'eval(^x)'
mago query 'is_null(^x)'
mago query 'str_contains(^haystack, ^needle)'
mago query '$^self == $^self'
```

### Scoping to part of the project

```sh
mago query 'eval(^x)' src/Controller src/Service
mago query 'dd(^x)' tests/
```

### Previewing rewrites

```sh
mago query '`is_null(^x)` => `^x === null`' --fix --dry-run
mago query '`count(^x) > 0` => `[] !== ^x`' --fix --dry-run
mago query '`shell_exec(^cmd, ^...rest)` => `exec(^...rest)`' --fix --dry-run
```

### Applying rewrites and reformatting

```sh
mago query '`is_null(^x)` => `^x === null`' --fix --format-after-fix
```

### Using line comments inside a pattern

```sh
mago query '
  // match assert() with no description
  assert(^cond)
'
```

## Exit codes

| Code | Meaning                                                                  |
|:-----|:-------------------------------------------------------------------------|
| `0`  | Command ran successfully; matches may or may not have been found.        |
| `1`  | A match at or above `--minimum-fail-level` was reported (non-fix mode).  |
| `2`  | Pattern failed to compile.                                               |

When `--fix` is passed and all rewrites apply cleanly, the exit code is `0` unless
`--fail-on-remaining` is set and unresolved issues remain.

## See also

- [Query overview](./overview.md) for the pattern language and recipes.
- [Shared Reporting and Fixing Options](/fundamentals/shared-reporting-options.md) for
  the flags `query` inherits from the rest of the toolchain.
