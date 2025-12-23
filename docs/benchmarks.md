---
title: Benchmarks
---

# Benchmarks

Performance is a core feature of **Mago**. Every component, from the parser to the analyzer, is designed to be as fast as possible.

We regularly benchmark Mago against other popular tools in the PHP ecosystem to ensure it remains the fastest toolchain available. The benchmarks below were run against the full `wordpress-develop` codebase.

## Our performance promise

At its core, Mago is built on a simple philosophy: **it must be the fastest.**

This is not just a goal; it's a guarantee. If any tool listed in our benchmarks ever outperforms Mago in a like-for-like comparison, we consider it a high-priority bug that needs to be fixed. Speed is a feature, and we promise to always deliver it.

## Formatter

This benchmark measures the time it takes to check the formatting of an entire codebase.

<BenchmarkChart
  title="Speed"
  :data="[
    { label: 'Mago', value: 0.416, highlight: true },
    { label: 'Pretty PHP', value: 35.83 }
  ]"
  unit="seconds"
/>

<BenchmarkChart
  title="Peak Memory (RSS)"
  :data="[
    { label: 'Mago', value: 663, highlight: true },
    { label: 'Pretty PHP', value: 150 }
  ]"
  unit="mb"
/>

## Linter

This benchmark measures the time it takes to lint an entire codebase.

<BenchmarkChart
  title="Speed"
  :data="[
    { label: 'Mago', value: 0.530, highlight: true },
    { label: 'Pint', value: 38.85 },
    { label: 'PHP-CS-Fixer', value: 53.28 }
  ]"
  unit="seconds"
/>

<BenchmarkChart
  title="Peak Memory (RSS)"
  :data="[
    { label: 'Mago', value: 504, highlight: true },
    { label: 'Pint', value: 78 },
    { label: 'PHP-CS-Fixer', value: 167 }
  ]"
  unit="mb"
/>

## Analyzer

This benchmark measures the time it takes to perform a full static analysis.

<BenchmarkChart
  title="Speed"
  :data="[
    { label: 'Mago', value: 3.88, highlight: true },
    { label: 'Psalm', value: 45.53 },
    { label: 'PHPStan', value: 120.35 }
  ]"
  unit="seconds"
/>

<BenchmarkChart
  title="Peak Memory (RSS)"
  :data="[
    { label: 'Mago', value: 930, highlight: true },
    { label: 'Psalm', value: 1464 },
    { label: 'PHPStan', value: 802 }
  ]"
  unit="mb"
/>

## Environment

- **Mago:** 1.0.0
- **Codebase:** `wordpress-develop@5b01d24`
- **Hardware:** MacBook Pro (Apple M1 Pro, 32GB RAM)
- **PHP:** 8.4.15 (Zend v4.4.15, Zend OPcache v8.4.15)

## A note on memory usage

You might notice that Mago sometimes uses more memory than other tools, especially on large codebases. This is a deliberate and fundamental design choice.

**Mago prioritizes your time over machine resources.**

To achieve its blazing-fast speeds, Mago uses per-thread arena allocators. Instead of asking the operating system for memory for every little object (which is slow), it reserves large chunks of memory upfront and then allocates objects within that arena with near-zero cost. The trade-off is that this can lead to a higher peak memory footprint.

We believe that in modern development environments, saving a developer several seconds—or even minutes—is a worthwhile trade for a temporary increase in RAM usage.
