---
title: The analyzer
---

# Mago's analyzer 🔬

**Mago**'s analyzer is a powerful static analysis engine that finds logical errors, type mismatches, and potential bugs in your code _before_ you run it. It's the core of Mago's ability to ensure your code is not just well-styled, but also correct and robust.

## Analyzer vs. linter: what's the difference?

While they both find issues, the analyzer and the linter operate at different levels of understanding.

- **The linter is a style editor**. It looks at the _structure_ of your code. It checks for stylistic issues, inconsistencies, and code smells (e.g., "this `if` statement has no `else`"). It doesn't know what your code _does_.

- **The analyzer is a fact-checker**. It builds a deep, semantic understanding of your entire codebase. It knows what types your functions return, what properties your classes have, and what exceptions can be thrown. It finds logical impossibilities (e.g., "you're calling a method that doesn't exist on this object").

:::tip Analogy
If your code were an essay, the **linter** would be the grammar and style checker, while the **analyzer** would be the editor who checks your facts and ensures your arguments are logical.
:::

## Key features

- **🚀 Blazing fast** — Written in Rust and highly parallelized to analyze large codebases in seconds.
- **Deep type inference** — Understands your code's types, even without full type hinting.
- **Comprehensive checks** — Catches a wide range of issues, from simple null pointer risks to complex logical errors.
- **Heuristic engine** — Provides advice on code quality issues that aren't strict errors but could indicate potential bugs.

## Dive in

- **[Configuration reference](/tools/analyzer/configuration-reference.md)**: see all the available options to customize the analysis.
- **[Command reference](/tools/analyzer/command-reference.md)**: a detailed guide to the `mago analyze` command and its flags.
