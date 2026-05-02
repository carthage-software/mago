+++
title = "Tools"
description = "Each Mago tool, what it does, and where to dig in."
nav_order = 10
nav_section = "Tools"
+++
# Tools

Mago is one binary that bundles four tools. They share configuration, parser, and runtime, so you can use any combination without paying for tools you do not run.

## [Formatter](/tools/formatter/overview/)

A deterministic code formatter. It produces stable, conventional output that follows [PER-CS](https://www.php-fig.org/per/coding-style/) by default and supports presets for PSR-12, Laravel, and Drupal style. No configuration roulette, no debate.

## [Linter](/tools/linter/overview/)

A curated catalogue of rules covering correctness, consistency, clarity, redundancy, safety, security, and a few other concerns. Most issues come with an automatic fix. Framework integrations layer on rules specific to Symfony, Laravel, PHPUnit, Doctrine, and others.

## [Analyzer](/tools/analyzer/overview/)

A static analysis engine that catches type errors and logic bugs before runtime. Compatible with Psalm and PHPStan annotations, with support for generics, conditional types, and flow narrowing.

## [Architectural guard](/tools/guard/overview/)

Enforces dependency rules and structural conventions. Useful when you want to forbid certain `use` paths, codify layer boundaries, or assert that code in one part of the project never imports code from another.
