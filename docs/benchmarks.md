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
    { label: 'Mago', value: 0.365, highlight: true },
    { label: 'Pretty PHP', value: 31.44 }
  ]"
  unit="seconds"
/>

<div class="memory-toggle">
<details>
<summary>Show Memory Usage</summary>
<div class="memory-content">

<BenchmarkChart
  title="Peak Memory (RSS)"
  :data="[
    { label: 'Mago', value: 632, highlight: true },
    { label: 'Pretty PHP', value: 155 }
  ]"
  unit="mb"
/>

</div>
</details>
</div>

## Linter

This benchmark measures the time it takes to lint an entire codebase.

<BenchmarkChart
  title="Speed"
  :data="[
    { label: 'Mago', value: 0.547, highlight: true },
    { label: 'Pint', value: 31.08 },
    { label: 'PHP-CS-Fixer', value: 49.64 }
  ]"
  unit="seconds"
/>

<div class="memory-toggle">
<details>
<summary>Show Memory Usage</summary>
<div class="memory-content">

<BenchmarkChart
  title="Peak Memory (RSS)"
  :data="[
    { label: 'Mago', value: 460, highlight: true },
    { label: 'Pint', value: 81 },
    { label: 'PHP-CS-Fixer', value: 123 }
  ]"
  unit="mb"
/>

</div>
</details>
</div>

## Analyzer

This benchmark measures the time it takes to perform a full static analysis (uncached) on the `wordpress-develop` codebase.

For comprehensive static analyzer benchmarks across multiple projects, versions, and categories, see the [**Static Analyzer Benchmarks Dashboard**](https://carthage-software.github.io/static-analyzers-benchmarks/) ([source](https://github.com/carthage-software/static-analyzers-benchmarks)).

<BenchmarkChart
  title="Speed"
  :data="[
    { label: 'Mago', value: 2.11, highlight: true },
    { label: 'Psalm', value: 11.37 },
    { label: 'Phan', value: 60.65 },
    { label: 'PHPStan', value: 61.85 }
  ]"
  unit="seconds"
/>

<div class="memory-toggle">
<details>
<summary>Show Memory Usage</summary>
<div class="memory-content">

<BenchmarkChart
  title="Peak Memory (RSS)"
  :data="[
    { label: 'Mago', value: 1017, highlight: true },
    { label: 'Psalm', value: 3958 },
    { label: 'PHPStan', value: 7119 },
    { label: 'Phan', value: 14295 }
  ]"
  unit="mb"
/>

</div>
</details>
</div>

## Environment

- **Hardware:** MacBook Pro (Apple M1 Pro, 32GB RAM), idle system
- **Codebase:** `wordpress-develop@5b01d24d8c5f2cfa4b96349967a9759e52888d03`
- **PHP:** 8.5.0 (Zend Engine v4.5.0, Zend OPcache v8.5.0)
- **Mago:** 1.9.1
- **Psalm:** 6.15.1
- **PHPStan:** 2.1.39
- **Phan:** 6.0.1
- **PHP-CS-Fixer:** 3.93.1
- **Pint:** 1.27.0
- **Pretty PHP:** 0.4.95

## A note on memory usage

Mago's memory usage varies depending on the task. In some cases, like static analysis, Mago actually uses **significantly less memory** than alternatives (3.5x less than Psalm). In other cases, such as linting or formatting, Mago may use more memory than single-threaded PHP tools.

This is a deliberate architectural choice. **Mago prioritizes your time over machine resources.**

To achieve its blazing-fast speeds, Mago uses per-thread arena allocators. Instead of asking the operating system for memory for every little object (which is slow), it reserves large chunks of memory upfront and then allocates objects within that arena with near-zero cost. This enables massive parallelism but can lead to a higher peak memory footprint for some operations.

We believe that in modern development environments, saving a developer several seconds, or even minutes, is a worthwhile trade for a temporary increase in RAM usage.

<style>
.memory-toggle {
  margin-top: 1.5rem;
}

.memory-toggle details {
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  background: var(--vp-c-bg-soft);
}

.memory-toggle summary {
  padding: 0 1rem;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  list-style: none;
}

.memory-toggle summary::-webkit-details-marker {
  display: none;
}

.memory-toggle summary::before {
  content: "▶";
  font-size: 0.7rem;
  transition: transform 0.2s;
}

.memory-toggle details[open] summary::before {
  transform: rotate(90deg);
}

.memory-toggle summary:hover {
  color: var(--vp-c-text-1);
  background: var(--vp-c-bg-mute);
  border-radius: 8px 8px 0 0;
}

.memory-toggle details:not([open]) summary:hover {
  border-radius: 8px;
}

.memory-toggle .memory-content {
  border-top: 1px solid var(--vp-c-divider);
}
</style>
