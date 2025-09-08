---
title: Benchmarks
---

# Benchmarks ⚡️

Performance is a core feature of **Mago**. Every component, from the parser to the analyzer, is designed to be as fast as possible.

We regularly benchmark Mago against other popular tools in the PHP ecosystem to ensure it remains the fastest toolchain available. The benchmarks below were run against the full `wordpress-develop` codebase.

## Our performance promise

At its core, Mago is built on a simple philosophy: **it must be the fastest.**

This is not just a goal; it's a guarantee. If any tool listed in our benchmarks ever outperforms Mago in a like-for-like comparison, we consider it a high-priority bug that needs to be fixed. Speed is a feature, and we promise to always deliver it.

## Formatter

This benchmark measures the time it takes to check the formatting of an entire codebase.

### Speed

| Tool | Time (mean ± σ) | Relative Speed |
| :--- | :--- | :--- |
| **Mago** | **362.3ms ± 4.6ms** | **1x** |
| Pretty PHP | 35.62s ± 0.06s | 98.32x slower |

### Resource usage

| Tool | Peak Memory (RSS) | CPU Cycles |
| :--- | :--- | :--- |
| **Mago** | 582 MB | **~9.4 Million** |
| Pretty PHP | **159 MB** | ~10.4 Million |

## Linter

This benchmark measures the time it takes to lint an entire codebase.

### Speed

| Tool | Time (mean ± σ) | Relative Speed |
| :--- | :--- | :--- |
| **Mago** | **745.8ms ± 7.1ms** | **1x** |
| Pint | 34.23s ± 0.05s | 45.89x slower |
| PHP-CS-Fixer | 41.81s ± 0.13s | 56.07x slower |

### Resource usage

| Tool | Peak Memory (RSS) | CPU Cycles |
| :--- | :--- | :--- |
| **Mago** | 541 MB | **~9.2 Million** |
| Pint | **74 MB** | ~9.8 Million |
| PHP-CS-Fixer | 77 MB | ~9.8 Million |

## Analyzer

This benchmark measures the time it takes to perform a full static analysis.

### Speed

| Tool | Time (mean ± σ) | Relative Speed |
| :--- | :--- | :--- |
| **Mago** | **3.86s ± 0.15s** | **1x** |
| Psalm | 45.42s ± 1.16s | 11.77x slower |
| PHPStan | 111.43s ± 0.45s | 28.88x slower |

### Resource usage

| Tool | Peak Memory (RSS) | CPU Cycles |
| :--- | :--- | :--- |
| **Mago** | 1.36 GB | **~9.8 Million** |
| Psalm | 1.52 GB | ~9.9 Million |
| PHPStan | **865 MB** | ~11.5 Million |

## Environment

- **Codebase:** `wordpress-develop@5b01d24`
- **Hardware:** MacBook Pro (Apple M1 Pro, 32GB RAM)
- **PHP:** 8.4.11 (Zend v4.4.11, Zend OPcache v8.4.11)

## A note on memory usage

You might notice that Mago sometimes uses more memory than other tools, especially on large codebases. This is a deliberate and fundamental design choice.

**Mago prioritizes your time over machine resources.**

To achieve its blazing-fast speeds, Mago uses per-thread arena allocators. Instead of asking the operating system for memory for every little object (which is slow), it reserves large chunks of memory upfront and then allocates objects within that arena with near-zero cost. The trade-off is that this can lead to a higher peak memory footprint.

We believe that in modern development environments, saving a developer several seconds—or even minutes—is a worthwhile trade for a temporary increase in RAM usage.
