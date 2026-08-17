+++
title = "Testing and debugging"
description = "Test extensions against a corpus, validate worker registration, and diagnose protocol failures."
nav_order = 20
nav_section = "Extensions"
nav_subsection = "Development"
+++
# Testing and debugging

Test an extension primarily through a small PHP corpus analyzed by the real Mago CLI. This exercises worker startup, registration, target matching, binary encoding, callbacks, and the resulting diagnostics together.

## Create a corpus

Keep a representative project inside the extension package:

```text
tests/corpus/
├── mago.toml
├── worker.php
└── src/
    ├── analyzer.php
    └── linter.php
```

The worker must load the package under test, not a copied implementation:

```php
<?php

declare(strict_types=1);

use Acme\Mago\AcmeExtension;
use Mago\Sdk\Worker;

require dirname(__DIR__, 2) . '/vendor/autoload.php';

(new Worker(
    AcmeExtension::create(),
))->run();
```

Configure that worker and restrict discovery to the corpus source:

```toml
[source]
paths = ["src"]

[linter]
baseline = "lint-baseline.toml"
baseline-variant = "strict"

[analyzer]
baseline = "analysis-baseline.toml"
baseline-variant = "strict"

[extension-hosts.acme]
command = ["php", "worker.php"]
```

Split the corpus into focused files that cover:

- matching and non-matching linter targets;
- analyzer behavior with and without a provider result;
- inferred types, argument validation, references, and entry points;
- invalid or incomplete framework metadata;
- interactions between multiple rules or providers.

## Lock the issue baseline

For a representative corpus, generate and commit separate linter and analyzer baseline files:

```sh
mago --workspace tests/corpus lint --generate-baseline
mago --workspace tests/corpus analyze --generate-baseline
```

The strict variant records issue codes and exact line ranges. CI should verify that the current diagnostics still match those files exactly:

```sh
mago --workspace tests/corpus lint --verify-baseline
mago --workspace tests/corpus analyze --verify-baseline
```

Verification fails when a diagnostic is added or removed. Regenerate a baseline only when an intentional extension change alters the expected result, then review the baseline diff before committing it.

For small, focused fixtures, inline expectations can be easier to review. Place `@mago-expect` immediately before every diagnostic that fixture should produce:

```php
<?php

declare(strict_types=1);

// @mago-expect lint:acme/no-eval
$result = eval($source);
```

When the named issue is emitted, Mago consumes it. If it disappears, Mago reports `unfulfilled-expect`; if the extension or Mago emits a new issue, that issue remains visible. A successful corpus run therefore has no output issues while still asserting every expected diagnostic by code and location.

When a corpus uses only inline expectations, run both tools and require zero issues:

```sh
mago --workspace tests/corpus lint --reporting-format count
mago --workspace tests/corpus analyze --reporting-format count
```

Both commands should report zero issues. Treat any non-zero count as a corpus regression.

For behavior whose success is silence, make the fixture fail observably when the extension is wrong. For example, pass a provider's inferred return value to a narrowly typed function so an incorrect type produces a native analyzer issue. Include a negative case where the provider returns `null` and Mago preserves native behavior.

For return/signature pairs, verify both argument validation and the final return type. For references and entry points, verify that unused-symbol diagnostics change only for the intended declaration.

## Unit test package logic separately

Unit tests remain useful for deterministic logic such as framework indexes, configuration validation, metadata mapping, and reducer aggregation. Use the test framework preferred by the package.

Do not replace corpus coverage with parser or type-comparison test doubles. Mago owns those semantics, so only a real CLI run proves that the extension integrates with them correctly.

## Validate registration

Before debugging callbacks, verify the host can start and advertise a stable registration:

```sh
mago extension validate
mago extension list
mago extension list --json
```

`extension validate` exercises host startup and linter registration. `extension list` reports hosts, logical extensions, and linter rules; it does not currently list analyzer plugins. Run an analyzer fixture to validate analyzer registration, plugin selection, targets, and capabilities.

Every worker in one pool must register identical extension identifiers, rule definitions, plugin selectors, targets, and capabilities. Do not make registration depend on a process ID, random value, request order, or mutable remote service.

## Inspect failures

Enable trace output:

```sh
MAGO_LOG=trace mago analyze --reporting-format count
```

Mago continuously retains the configured tail of worker standard error. A non-empty tail is appended when the worker failure or disconnect path captures it, but startup failures, request timeouts, and remote callback errors can be reported without that tail. Log concise context to `STDERR`; never write application output, a byte-order mark, or debug dumps to `STDOUT`.

Useful isolation switches are:

- `--no-extensions` to confirm the failure is extension-related;
- a disabled host in a configuration overlay;
- `[analyzer].disable-default-plugins` plus an explicit plugin selector;
- `lint --only vendor/rule` for one external rule;
- `workers = 1` to make process-local behavior reproducible.

## Timeouts and cancellation

The host's `request-timeout-ms` covers an outer callback and its nested metadata or comparison requests. When Mago cancels work, `CancellationTokenInterface` becomes cancelled and `throwIfCancelled()` raises `CancelledException`.

Check cancellation in long scans and aggregation loops. `subscribe()` integrates cancellation with asynchronous work; always `unsubscribe()` a retained subscription after completion.

Do not catch `CancelledException` merely to continue expensive work. Mago removes cancelled pending work and ignores a response that arrives after cancellation; callback wrappers may still convert a thrown exception into a protocol error before that response is discarded.

## Test failure policy

Provider failures should leave native analysis usable. Test that a throwing or disconnected optional provider falls back rather than inventing a type. Lifecycle failures should fail the analysis operation with host or plugin context; they are not ordinary source diagnostics.

Also test malformed framework data. An extension should return `null` or report a precise issue rather than sending invalid SDK values that turn a user-code problem into a protocol error.

## Cross-platform checks

Run integration tests on Linux, macOS, and Windows. The SDK uses redirected standard input on Unix-like systems and an authenticated loopback input transport where Windows cannot make redirected PHP standard input non-blocking. Extension entrypoints should call `Worker::run()` normally and never implement transport selection themselves.

Avoid shell-only commands in `command`: Mago performs no shell parsing. Use an executable and literal argument list that works on every supported platform, or document platform-specific host configuration explicitly.

## Incremental and repeated analysis

Workers can serve repeated analysis generations during one command. Tests should ensure that:

- codebase-scan state is cleared on `firstBatch`;
- provider caches do not leak stale project metadata across generations;
- after-file references are replaced, not duplicated;
- reducer state represents one intended worker lifetime;
- exact in-memory source is used instead of rereading changed files from disk.
