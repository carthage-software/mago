+++
title = "Formatter"
description = "What the formatter does and how it produces deterministic output."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Formatter

A deterministic formatter for PHP. Run it on a file and you get the same output regardless of the file's current style. Stop arguing about whitespace, start reading code.

## How it works

Mago borrows the parse-and-reprint approach used by Prettier, `rustfmt`, and Black:

1. Parse the source into an AST.
2. Discard the original formatting (whitespace, line breaks, indentation, all of it).
3. Reprint the AST from scratch according to a fixed set of rules, [PER-CS](https://www.php-fig.org/per/coding-style/) by default.

The output is identical for a given AST regardless of the input style. The runtime behaviour of the code is preserved exactly: the AST round-trips, only the surface representation changes.

## What you get

- **One consistent style** across the entire project. The formatter is opinionated by design.
- **PER-CS by default**, with optional presets for PSR-12, Laravel, and Drupal style.
- **Safe.** The formatter is constrained to changes that cannot alter program behaviour.
- **Fast.** A Rust core and an arena-backed pipeline keep the format pass well under a second on most projects.

## Where to next

- [Usage](/tools/formatter/usage/): how to run `mago format`.
- [Format ignore](/tools/formatter/format-ignore/): pragmas to skip formatting for files, regions, or individual statements.
- [Configuration reference](/tools/formatter/configuration-reference/): every option you can set.
- [Command reference](/tools/formatter/command-reference/): every flag `mago format` accepts.
