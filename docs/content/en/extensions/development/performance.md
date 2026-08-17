+++
title = "Performance"
description = "Keep external extensions close to Mago's native throughput."
nav_order = 10
nav_section = "Extensions"
nav_subsection = "Development"
+++
# Extension performance

External extensions add process startup, binary serialization, and worker callback work. Mago's API is designed to avoid paying those costs for irrelevant files and nodes. Performance therefore depends as much on registration shape as on language-level micro-optimizations.

## Measure the complete command

Benchmark the real project and command users run. Compare the same release binary with extensions enabled and disabled:

```sh
hyperfine --warmup 3 \
  'mago --workspace /path/to/project --no-extensions analyze --reporting-format count' \
  'mago --workspace /path/to/project analyze --reporting-format count'
```

For linter rules, benchmark both the complete configured rule set and one external rule:

```sh
hyperfine --warmup 3 \
  'mago --workspace /path/to/project --no-extensions lint --reporting-format count' \
  'mago --workspace /path/to/project lint --only acme/rule --reporting-format count'
```

Use a representative application, not only a small clean library. Report Mago version, PHP version, worker configuration, logical CPU count, project revision, command, warmup count, and distribution rather than one best timing.

## Trace the pipeline

Set `MAGO_LOG=trace` to expose command-stage and extension telemetry:

```sh
MAGO_LOG=trace mago analyze --reporting-format count
```

Trace output includes host startup, active worker counts, dispatch and provider timing summaries, protocol activity, and slow callback information without logging every request and response body. Redirect it to a file when comparing runs:

```sh
MAGO_LOG=trace mago analyze --reporting-format count 2>mago-trace.log
```

An extension may write its own diagnostics to standard error. Standard output is reserved exclusively for protocol frames.

## Optimize selection before callbacks

The fastest callback is one Mago never sends.

- Linter rules should target exact `NodeKind` cases.
- Function, method, property, and class providers should use the narrowest targets.
- Use semantic method and class-like hooks instead of scanning broad syntax and querying metadata afterward.
- Codebase scan hooks should select only paths that can contribute framework data.
- Entry points and attributed entry points should be registered for native matching.
- Issue filters should declare only the native codes they can remove.

Avoid a universal target followed by a large PHP `if` chain. It inflates snapshot creation, IPC, decoding, and callback dispatch before the common rejection.

## Minimize snapshots

Targeted analysis hooks explicitly request data. Each extra `FileAnalysisRequirement` has a cost:

- source text copies file bytes;
- target subtrees retain and encode more syntax;
- receiver and argument types serialize complete type information;
- all expression types can dominate large files.

Request only values the hook reads. Use lazy `FileAnalysis` methods for uncommon branches rather than embedding expensive data for every matching file.

## Batch nested requests

Metadata, expression types, references, and type comparisons cross back from PHP to Mago as nested requests. Their singular methods are ergonomic, not an invitation to create loops of round trips.

Prefer:

```php
$classes = $codebase->getMultipleClasses($names);
$exists = $codebase->checkMultipleMethodsExist($methods);
$types = $analysis->getMultipleExpressionTypes($spans);
$incoming = $references->getMultipleReferencesTo($symbols);
$results = $types->compareMultiple($comparisons);
```

The SDK caches supported immutable lookups and preserves batch input order. `compareMultiple()`, expression-type batches, and reference-graph batches also deduplicate repeated unresolved keys before crossing the protocol boundary.

## Worker pools

The default `workers = 0` is adaptive. Mago starts at most three processes and may grow the pool toward the global thread count when sustained request contention and observed callback time justify more processes. An analyzer host with parallel after-file or targeted work may proactively reach half of that capacity, rounded up, before dispatch. This avoids eagerly starting every language runtime for sparse callbacks.

A positive worker count fixes the pool size. Use it for measured constraints, not as a default tuning ritual. While Mago threads wait for responses, the operating system can schedule worker processes on the same CPUs.

For the PHP SDK, CPU-bound work needs multiple processes for parallelism because Fibers provide concurrency only. SDKs in other languages retain their own within-process execution model, but should still avoid oversubscribing the CPUs Mago uses.

Cooperative Revolt I/O can interleave inside one worker, but do not turn hot semantic providers into network clients. External state also makes results slow, fragile, and difficult to memoize.

## Process-local state

Construct expensive immutable configuration once in the worker entrypoint or plugin object, not per callback. State is local to one worker process.

Codebase scan hooks broadcast selected input to each worker, allowing each process to build its own equivalent index. Keep indexes compact and keyed for the provider's common lookup path.

Use a `WorkerReducer` only when process-local results must be combined for an external action at shutdown. It is not a live cross-worker synchronization mechanism.

## Memoization

`PluginRegistry::enableProviderMemoization()` can remove repeated function and method return-type provider requests, including callable-signature requests. Enable it only when every such provider in the plugin is deterministic and independent of locations, order, and mutable state. One unsafe provider invalidates the contract for the whole registry.

Cache your own pure PHP computations when keys are compact and hit rates justify retained memory. Do not duplicate the specific lookup caches already provided by `Codebase`, `TypeComparator`, `FileAnalysis`, or `SymbolReferences`; note that `FileAnalysis::getAllExpressionTypes()` intentionally performs a request on every call.

## PHP hot-path guidance

- Return immediately on the common non-match.
- Avoid reflection, regular expressions, and filesystem reads in provider callbacks.
- Prefer direct DTO field access to rebuilding arrays.
- Avoid reparsing PHP source; use Mago's syntax and semantic snapshots.
- Reuse immutable `Type` and configuration values.
- Check cancellation in long loops, not on every trivial operation.
- Keep worker startup free of unnecessary framework bootstrapping.

Do not depend on classes under `Mago\Sdk\Internal`. They are optimized protocol machinery and may change without compatibility guarantees.

## Diagnose regressions

1. Compare with `--no-extensions`.
2. Run `mago extension list --json` and verify the configured hosts, logical extensions, and linter rules. Analyzer capabilities are not listed there; exercise them with an analyzer fixture.
3. Disable individual analyzer plugins or select one linter rule.
4. Inspect trace timing for startup, snapshots, nested requests, and callbacks.
5. Count actual target matches; an unexpectedly broad target is often the cause.
6. Remove unnecessary requirements and batch repeated queries.
7. Rebenchmark on the same revision and worker count.
