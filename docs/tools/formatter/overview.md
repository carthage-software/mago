---
title: The formatter
---

# Mago's formatter ✨

**Mago**'s formatter is a powerful, opinionated code formatter for PHP that ensures your entire codebase adheres to a single, consistent style.

Its primary goal is to end debates over code style. By automating the formatting process, it allows you and your team to stop worrying about whitespace and focus on what truly matters: building great software.

## How it works

Mago takes a "parse-and-reprint" approach, inspired by modern formatters like Prettier and `rustfmt`.

1.  It first parses your PHP code into a detailed Abstract Syntax Tree (AST).
2.  It then **throws away your original formatting**, including all newlines, spacing, and indentation.
3.  Finally, it **reprints the AST from scratch** according to its own set of consistent, [PER-CS](https://www.php-fig.org/per/coding-style/)-compliant rules.

This process guarantees that the output is always 100% consistent, regardless of the input style. Most importantly, it does this without ever changing the behavior of your code.

## Key features

- **Blazing fast** — Written in Rust for maximum performance, making it one of the fastest PHP formatters available.
- **Opinionated & consistent** — Ends style debates by enforcing a single, unified coding style across your entire project.
- **PER-CS compliant** — Follows the widely accepted [PER-CS](https://www.php-fig.org/per/coding-style/) coding standard.
- **Safe** — The formatter is designed to never alter the runtime behavior of your code.

## Dive in

- **[Usage](/tools/formatter/usage.md)**: learn how to run the formatter from the command line.
- **[Format Ignore](/tools/formatter/format-ignore.md)**: learn how to selectively disable formatting for files, regions, or individual statements.
- **[Configuration reference](/tools/formatter/configuration-reference.md)**: see all the available options to customize the formatter's behavior.
- **[Command reference](/tools/formatter/command-reference.md)**: a detailed guide to the `mago format` command and its flags.
