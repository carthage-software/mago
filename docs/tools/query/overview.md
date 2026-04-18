---
title: Query
outline: deep
---

# Mago's query engine 🔍 (experimental)

`mago query` is a structural search-and-rewrite tool for PHP, inspired by
[GritQL](https://docs.grit.io) and [ast-grep](https://ast-grep.github.io). You describe
the AST shape you want to find with a small pattern language, and Mago walks every
file in your workspace in parallel to produce matches and, optionally, rewrites.

::: warning Experimental
The query engine is experimental. Pattern syntax, output, and CLI flags may change
without warning between releases. Do not depend on it for reproducible tooling yet.
:::

## What it can do

- Find every call to a function, method, or language construct.
- Capture sub-expressions as named metavariables and assert they're equal across uses.
- Match statement-, class-member-, and function-shaped patterns, not just expressions.
- Apply template rewrites to every match, previewed as a diff or applied in place.
- Run at the same parallel scale as `mago lint`, `mago format`, and `mago analyze`.
- Report every match as a `help`-level [`Issue`](/fundamentals/shared-reporting-options.md)
  with code `query-match`, so it flows through every reporting format
  (`rich`, `medium`, `short`, `json`, `github`, `gitlab`, ...) and honours every
  reporting flag.

## 60-second tour

```sh
# Find every call to is_null, anywhere in the workspace.
mago query 'is_null(^x)'

# Preview the rewrite is_null($x) -> ($x === null) as a unified diff.
mago query '`is_null(^x)` => `^x === null`' --fix --dry-run

# Apply it.
mago query '`is_null(^x)` => `^x === null`' --fix

# Only touch a specific subdirectory.
mago query 'eval(^x)' src/Controller

# Emit JSON for a script, CI check, or editor plugin.
mago query 'eval(^x)' --reporting-format json
```

## Metavariables

| Form       | Meaning                                                                  |
|:-----------|:-------------------------------------------------------------------------|
| `^name`    | Matches any single node / token and binds it to `name`.                  |
| `^...name` | Matches zero or more sibling nodes (a sequence) and binds the slice.     |
| `^...`     | Anonymous sequence match: absorbs remaining siblings without binding.    |
| `$^name`   | Matches a PHP variable whose name is the metavariable (e.g. `$foo`).     |

Capture names must be consistent across uses within one pattern:
`strcmp(^x, ^x)` only matches calls where both arguments are the same text.

## Pattern syntax

Two forms are accepted:

- **Bare snippet** (no backticks): the pattern is a single PHP-ish expression, for
  example `is_null(^x)`. Best for simple searches.
- **Surface grammar** (contains backticks): a GritQL subset wrapped around one or
  more backticked PHP snippets. Required for rewrites and boolean combinators.

### Surface-grammar operators

| Form                                | Meaning                                                 |
|:------------------------------------|:--------------------------------------------------------|
| `` `lhs` => `rhs` ``                | Rewrite every match of `lhs` to `rhs`.                  |
| `P <: Q`                            | `P` is a subtype of `Q` (both must match).              |
| `not P`, `!P`                       | Negation.                                               |
| `contains P`                        | Matches any node whose subtree contains a match of `P`. |
| `within P`                          | Matches when the current node is inside a match of `P`. |
| `maybe P`                           | Matches whether or not `P` matches (useful in blocks).  |
| `bubble P`                          | Scope-reset: lifts `P` into a fresh variable scope.     |
| `P where { clauses… }`              | Conjunction: `P` plus every clause must hold.           |
| `or { A, B, … }`, `and { A, B, … }` | Explicit boolean combinators.                           |
| `// line comment`                   | Ignored by the parser.                                  |

### Rewrite templates

The right-hand side of `=>` is a **template**, not another pattern. Inside a
backticked template body:

- `^name` splices the captured text of that metavariable.
- `^...name` splices a captured sequence, joining its elements with `, `.
- Everything else is copied verbatim.

A template may only reference variables that the LHS actually binds. A reference
to a name that wasn't bound on the left is a compile-time error:

```
$ mago query '`shell_exec(^arg)` => `exec(^nope)`'
Failed to compile pattern: surface grammar error:
  right-hand side references `^nope`, which is not bound by the left-hand side of the rewrite
```

### `=>` inside snippets is ordinary PHP

The surface-level `=>` is only recognised **between** backticked snippets. Inside a
snippet body it is the ordinary PHP `key => value` token, so the following pattern
matches array entries whose key is the literal `'f'`:

```sh
mago query "['f' => ^value]"
```

## Recipes

Every recipe below is a complete command you can run from any mago workspace root.

### Replace loose equality with strict equality

```sh
mago query '`^a == ^b` => `^a === ^b`' --fix --dry-run
```

```diff
- if ($x == 1) { /* … */ }
+ if ($x === 1) { /* … */ }
```

### Replace `is_null($x)` with `$x === null`

```sh
mago query '`is_null(^x)` => `^x === null`' --fix --dry-run
```

```diff
- if (is_null($value)) { return; }
+ if ($value === null) { return; }
```

### Replace `strlen($s) > 0` with `$s !== ''`

```sh
mago query '`strlen(^s) > 0` => `^s !== ""`' --fix --dry-run
```

```diff
- if (strlen($name) > 0) { /* … */ }
+ if ($name !== "") { /* … */ }
```

### Drop the first argument and forward the rest

Variadic sequence metavariables splice the remaining arguments back into the
rewrite:

```sh
mago query '`shell_exec(^cmd, ^...rest)` => `exec(^...rest)`' --fix --dry-run
```

```diff
- shell_exec('ls', '-la', $env);
- shell_exec('whoami');
- shell_exec('echo', $msg);
+ exec('-la', $env);
+ exec();
+ exec($msg);
```

Note that `shell_exec('whoami')` rewrites to `exec()` with no trailing comma: an
empty captured sequence splices as nothing.

### Reorder arguments

```sh
mago query '`wrap(^first, ^...middle, ^last)` => `unwrap(^last, ^...middle, ^first)`'
```

### Find every `eval()` call regardless of nesting

```sh
mago query 'eval(^x)'
```

### Find loose equality *except* when the right-hand side is `null`

```sh
mago query '`^a == ^b` where { not `^a == null`, not `null == ^b` }'
```

### Only match assertions that still use loose equality

```sh
mago query 'assert(^a == ^b)'
```

## Reporting and fixing flags

Matches flow through the same pipeline as `mago lint`, so every reporting and
fixing flag works identically. See
[Shared Reporting and Fixing Options](/fundamentals/shared-reporting-options.md)
for the full catalogue.

The two flags you'll reach for most often with `query` are:

- `--fix`, to apply rewrites in place.
- `--fix --dry-run` (alias `--diff`), to preview rewrites as a coloured unified
  diff, identical in format to `mago fmt --diff`, without touching files.

## Troubleshooting

### "No matches for pattern" but I expected some

1. If your pattern contains a top-level `=>` but no backticks, Mago treats the
   whole string as one PHP snippet and your `=>` silently becomes the PHP
   key-value token. **Solution**: wrap both sides in backticks:
   `` `lhs` => `rhs` ``.
2. Check that your captures are consistent. `strcmp(^x, ^x)` only matches calls
   where both arguments render to the same text.
3. Make sure the target path contains PHP files and isn't in your workspace's
   `excludes`.

### Compile error: "right-hand side references `^foo`, which is not bound"

The RHS template used a metavariable that the LHS pattern never introduced. Add
the variable to the LHS or drop the reference from the RHS.

### Rewrite output contains a literal `^...name`

Your RHS is outside a backtick-delimited template. `^...name` is only expanded
when it appears inside a `` ` … ` `` snippet body. Put the whole RHS in
backticks.

## Dive in

- **[Command Reference](./command-reference.md)**: every flag and argument.
- **[Shared Reporting and Fixing Options](/fundamentals/shared-reporting-options.md)**:
  the reporting flags that `query`, `lint`, `analyze`, `guard`, and `ast` share.
