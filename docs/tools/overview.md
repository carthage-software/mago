---
title: "Tools Overview"
outline: deep
---

# Tools overview

**Mago** is a comprehensive toolchain, not just a single utility. It's composed of several powerful, high-performance components that work together to improve your PHP code.

This section provides a detailed guide for each tool.

### [Formatter](/tools/formatter/overview.md)

The **formatter** is an opinionated code formatter that ensures your entire codebase adheres to a single, consistent style based on PSR-12. It's designed to end debates over code style forever.

### [Linter](/tools/linter/overview.md)

The **linter** is a blazing-fast tool for finding stylistic issues, inconsistencies, and code smells. It helps you maintain a clean and readable codebase with minimal effort.

### [Analyzer](/tools/analyzer/overview.md)

The **analyzer** is a powerful static analysis engine that finds logical errors, type mismatches, and potential bugs in your code _before_ you run it. It's the core of Mago's ability to ensure your code is correct and robust.

### [Architectural Guard](/tools/guard/overview.md)

The **architectural guard** is a powerful architectural validation utility for enforcing boundaries and coding standards within a PHP project.

### [Lexer & parser](/tools/lexer-parser/overview.md)

At the heart of Mago lies its high-performance Lexer and Parser. These components turn your raw PHP source code into a structured Abstract Syntax Tree (AST). The `mago ast` command provides a powerful way to inspect this structure for debugging and learning.
