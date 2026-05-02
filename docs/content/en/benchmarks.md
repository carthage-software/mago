+++
title = "Benchmarks"
description = "How Mago compares to other PHP tooling, and the trade-offs we made to get there."
nav_order = 20
nav_section = "Reference"
+++
# Benchmarks

Mago is built to be the fastest PHP toolchain. Every component, from the parser to the analyzer, was designed around that constraint.

We benchmark Mago against the rest of the PHP ecosystem on a regular basis. The numbers and history are on the dedicated dashboard:

- [PHP Toolchain Benchmarks](https://carthage-software.github.io/php-toolchain-benchmarks/?project=psl&kind=Analyzers)
- [Source repository](https://github.com/carthage-software/php-toolchain-benchmarks)

The dashboard runs three benchmarks against several real PHP codebases:

- Formatter: time to check the formatting of an entire codebase.
- Linter: time to lint an entire codebase.
- Analyzer: time to perform a full static analysis from a cold cache.

## The performance promise

Speed is not a goal, it is a guarantee. If any tool listed in the benchmarks ever outperforms Mago on a like-for-like comparison, we treat that as a high-priority bug.

## A note on memory

Memory use varies with the task. For static analysis Mago typically uses less memory than alternatives (around 3.5x less than Psalm in our runs). For lint and format, Mago can use more memory than a single-threaded PHP tool.

This is deliberate. Mago prioritises your time over machine resources.

To get the speeds we want, Mago uses per-thread arena allocators. Instead of asking the operating system for memory for every small object, it reserves large chunks up front and allocates within them at near-zero cost. That enables heavy parallelism, at the cost of a higher peak memory footprint on some workloads. We think trading a few hundred megabytes of RAM for several seconds (or minutes) of developer time is a sensible deal.
