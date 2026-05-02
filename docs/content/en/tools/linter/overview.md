+++
title = "Linter"
description = "What the linter does, how it differs from the analyzer, and where to read next."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Linter

Mago's linter is a curated catalogue of rules that catch stylistic issues, inconsistencies, and code smells. Most issues ship with an automatic fix, so you can clean up large codebases with a single command.

## Linter vs analyzer

Both tools find issues, but they operate at different levels.

The **linter** looks at the *shape* of your code. It enforces team conventions, flags redundant constructs, and suggests more modern syntax. It does not need to know what your code does at runtime, only what it looks like in source.

The **analyzer** builds a semantic model of your entire codebase. It knows what types functions return, what properties classes have, and what can throw. It finds logical impossibilities like calling a method that does not exist on the type at hand.

If your code were an essay, the linter is the grammar pass and the analyzer is the fact-check.

## The semantic checker

Mago processes files in three stages: parse, semantic check, lint.

The parser is intentionally tolerant. It can read syntax the standard PHP compiler would reject, including features from a future PHP version. The semantic checker is the second stage that catches errors the tolerant parser lets through but PHP would treat as fatal:

- Invalid enum backing types like `enum Foo: array {}`.
- Features that are not available in the configured PHP version, for example property hooks before PHP 8.4.

Run just the parser and semantic checker with `--semantics`:

```sh
mago lint -s
```

This is a faster, more thorough replacement for `php -l` and a low-friction way to introduce Mago to a codebase before turning on the full rule catalogue.

## What the linter offers

- **Speed.** A Rust core and an arena-backed pipeline keep the lint pass well under a second on most projects.
- **Per-rule configuration.** Every rule can be enabled, disabled, or have its severity adjusted. Some rules carry their own options.
- **Auto-fixes.** Many rules ship a safe fix; pass `--fix` and Mago rewrites the affected files. Less safe categories are gated behind explicit flags.
- **Framework integrations.** Optional rule sets specific to Symfony, Laravel, PHPUnit, Doctrine, WordPress, and a long tail of others. Enabled per project, listed on the [integrations page](/tools/linter/integrations/).

## Where to next

- [Usage](/tools/linter/usage/): how to run `mago lint`.
- [Integrations](/tools/linter/integrations/): enable framework-specific checks.
- [Rules](/tools/linter/rules/): the full reference for every rule.
- [Configuration reference](/tools/linter/configuration-reference/): every option Mago accepts under `[linter]`.
- [Command reference](/tools/linter/command-reference/): every flag `mago lint` accepts.
