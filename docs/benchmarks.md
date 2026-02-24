---
title: Benchmarks
---

<script setup>
import { ref, computed, onMounted } from 'vue'

const DATA_URL = 'https://carthage-software.github.io/php-toolchain-benchmarks/latest.json'
const PROJECT_LABELS = { wordpress: 'WordPress', psl: 'PSL', magento: 'Magento' }

const raw = ref(null)
const loading = ref(true)
const error = ref(null)
const selectedProject = ref('wordpress')

onMounted(async () => {
  try {
    const res = await fetch(DATA_URL)
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    raw.value = await res.json()
  } catch (e) {
    error.value = 'Failed to load benchmark data. Please try refreshing the page.'
  } finally {
    loading.value = false
  }
})

function parseVersion(v) {
  return v.split('.').map(Number)
}

function compareVersions(a, b) {
  const pa = parseVersion(a)
  const pb = parseVersion(b)
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    if ((pa[i] || 0) !== (pb[i] || 0)) return (pa[i] || 0) - (pb[i] || 0)
  }
  return 0
}

function getLatestVersions(toolsObj) {
  const groups = {}
  for (const [name, measurements] of Object.entries(toolsObj)) {
    const match = name.match(/^(.+?)\s+([\d.]+)$/)
    if (!match) continue
    const [, baseName, version] = match
    if (!groups[baseName] || compareVersions(version, groups[baseName].version) > 0) {
      groups[baseName] = { version, fullName: name, measurements }
    }
  }
  return groups
}

function getStats(measurements) {
  const valid = measurements.filter(m => !m.timed_out)
  if (!valid.length) return null
  return {
    time: valid.reduce((s, m) => s + m.mean, 0) / valid.length,
    memory: Math.max(...valid.map(m => m.memory_mb || 0)),
  }
}

function cleanLabel(baseName, category) {
  if (category === 'Formatter') return baseName.replace('Mago Fmt', 'Mago')
  if (category === 'Linter') return baseName.replace('Mago Lint', 'Mago')
  return baseName
}

function chartData(category, metric) {
  if (!raw.value) return []
  const project = raw.value.projects[selectedProject.value]
  if (!project || !project[category]) return []

  const latest = getLatestVersions(project[category])
  const entries = []
  for (const [baseName, { measurements }] of Object.entries(latest)) {
    const stats = getStats(measurements)
    if (!stats) continue
    const isMago = baseName.startsWith('Mago')
    entries.push({
      label: cleanLabel(baseName, category),
      value: metric === 'time' ? Math.round(stats.time * 1000) / 1000 : Math.round(stats.memory),
      highlight: isMago,
      isMago,
    })
  }

  const mago = entries.filter(e => e.isMago)
  const rest = entries.filter(e => !e.isMago).sort((a, b) => a.value - b.value)
  return [...mago, ...rest]
}

const formatterSpeed = computed(() => chartData('Formatter', 'time'))
const formatterMemory = computed(() => chartData('Formatter', 'memory'))
const linterSpeed = computed(() => chartData('Linter', 'time'))
const linterMemory = computed(() => chartData('Linter', 'memory'))
const analyzerSpeed = computed(() => chartData('Cold', 'time'))
const analyzerMemory = computed(() => chartData('Cold', 'memory'))
const projects = computed(() => raw.value ? Object.keys(raw.value.projects) : [])
const aggregationDate = computed(() => raw.value ? raw.value['aggregation-date'] : '')
</script>

# Benchmarks

Performance is a core feature of **Mago**. Every component, from the parser to the analyzer, is designed to be as fast as possible.

We regularly benchmark Mago against other popular tools in the PHP ecosystem to ensure it remains the fastest toolchain available. The data below is fetched live from the [**PHP Toolchain Benchmarks**](https://carthage-software.github.io/php-toolchain-benchmarks/?project=psl&kind=Analyzers) dashboard ([source](https://github.com/carthage-software/php-toolchain-benchmarks)).

## Our performance promise

At its core, Mago is built on a simple philosophy: **it must be the fastest.**

This is not just a goal; it's a guarantee. If any tool listed in our benchmarks ever outperforms Mago in a like-for-like comparison, we consider it a high-priority bug that needs to be fixed. Speed is a feature, and we promise to always deliver it.

<div v-if="loading" class="benchmark-status">Loading benchmark data...</div>
<div v-else-if="error" class="benchmark-status benchmark-error">{{ error }}</div>
<template v-else>

<div class="project-selector">
  <button
    v-for="p in projects"
    :key="p"
    :class="['project-btn', { active: p === selectedProject }]"
    @click="selectedProject = p"
  >{{ PROJECT_LABELS[p] || p }}</button>
  <span class="benchmark-date">Updated: {{ aggregationDate }}</span>
</div>

## Formatter

This benchmark measures the time it takes to check the formatting of an entire codebase.

<BenchmarkChart title="Speed" :data="formatterSpeed" unit="seconds" />

<BenchmarkChart title="Peak Memory (RSS)" :data="formatterMemory" unit="mb" />

## Linter

This benchmark measures the time it takes to lint an entire codebase.

<BenchmarkChart title="Speed" :data="linterSpeed" unit="seconds" />

<BenchmarkChart title="Peak Memory (RSS)" :data="linterMemory" unit="mb" />

## Analyzer

This benchmark measures the time it takes to perform a full static analysis (uncached).

<BenchmarkChart title="Speed" :data="analyzerSpeed" unit="seconds" />

<BenchmarkChart title="Peak Memory (RSS)" :data="analyzerMemory" unit="mb" />

</template>

## A note on memory usage

Mago's memory usage varies depending on the task. In some cases, like static analysis, Mago actually uses **significantly less memory** than alternatives (3.5x less than Psalm). In other cases, such as linting or formatting, Mago may use more memory than single-threaded PHP tools.

This is a deliberate architectural choice. **Mago prioritizes your time over machine resources.**

To achieve its blazing-fast speeds, Mago uses per-thread arena allocators. Instead of asking the operating system for memory for every little object (which is slow), it reserves large chunks of memory upfront and then allocates objects within that arena with near-zero cost. This enables massive parallelism but can lead to a higher peak memory footprint for some operations.

We believe that in modern development environments, saving a developer several seconds, or even minutes, is a worthwhile trade for a temporary increase in RAM usage.

<style>
.benchmark-status {
  padding: 2rem;
  text-align: center;
  color: var(--vp-c-text-2);
  font-size: 0.95rem;
}

.benchmark-error {
  color: var(--vp-c-danger-1);
}

.project-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 1.5rem 0;
  flex-wrap: wrap;
}

.project-btn {
  padding: 0.4rem 1rem;
  border: 1px solid var(--vp-c-divider);
  border-radius: 20px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-2);
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 500;
  transition: all 0.2s;
}

.project-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.project-btn.active {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
  color: #fff;
}

.benchmark-date {
  margin-left: auto;
  font-size: 0.8rem;
  color: var(--vp-c-text-3);
}
</style>
