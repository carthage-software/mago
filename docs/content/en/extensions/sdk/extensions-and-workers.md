+++
title = "Extensions and workers"
description = "Define logical extensions and run them in command-scoped PHP workers."
nav_order = 10
nav_section = "Extensions"
nav_subsection = "PHP SDK"
+++
# Extensions and workers

`Mago\Sdk\Extension` describes one logical package. `Mago\Sdk\Worker` registers one or more of those packages and serves Mago requests until shutdown.

## Defining an extension

An extension package should expose a factory that owns its complete registration:

```php
namespace Vendor\Framework\Mago;

use Vendor\Framework\Mago\Analyzer\FrameworkAnalyzerPlugin;
use Vendor\Framework\Mago\Linter\Rules\FrameworkRule;
use Vendor\Framework\Mago\Worker\MetricsReducer;
use Mago\Sdk\Extension;

final class FrameworkExtension
{
    private function __construct() {}

    public static function create(bool $strict = false): Extension
    {
        $metrics = new MetricsReducer();

        return new Extension(
            identifier: 'vendor/framework',
            name: 'Framework support',
            version: '2.3.0',
            linterRules: [new FrameworkRule($strict)],
            analyzerPlugins: [new FrameworkAnalyzerPlugin($metrics)],
            workerReducer: $metrics,
        );
    }
}
```

Keep the identifier, version, default capability set, and shared object wiring inside the package. Factory parameters expose only intentional typed options. A consuming project should not instantiate `Extension` for a third-party package or manually choose which of its rules and plugins to register.

| Argument | Contract | Purpose |
| :--- | :--- | :--- |
| `identifier` | non-empty string | Stable, globally unique extension identifier. Use a vendor-qualified value. |
| `name` | non-empty string | Human-readable display name. |
| `version` | non-empty string | Extension package version reported during registration. |
| `linterRules` | `list<Rule>` | Custom linter rules exposed by the extension. |
| `analyzerPlugins` | `list<Plugin>` | Analyzer plugins exposed by the extension. |
| `workerReducer` | `?WorkerReducer` | Optional terminal aggregation for process-local state. |

Extension identifiers and analyzer plugin selectors are compared ASCII case-insensitively. Linter rule codes are case-sensitive and must be unique within an extension, across every enabled host, and against Mago's native rule codes. Analyzer plugin identifiers and aliases must not collide with another external or native plugin selector. Mago also verifies that every process in one host pool advertises the same registration.

## Running a worker

```php
use Vendor\Framework\Mago\FrameworkExtension;
use Mago\Sdk\Worker;

(new Worker(
    FrameworkExtension::create(strict: true),
))->run();
```

`run()` owns the process protocol loop. It normally reads from the SDK-selected input transport and writes frames to standard output. The optional input/output resource arguments exist for embedding and tests; production entrypoints should use the defaults.

Do not write application output to standard output. Use standard error for debugging:

```php
fwrite(STDERR, "Framework extension initialized.\n");
```

The worker captures PHP output-buffer output and redirects throwable details to standard error. Direct writes to `STDOUT`, output emitted before `run()` installs its buffer, and native output that bypasses PHP's output buffer can still corrupt protocol frames.

## Hosting several extensions

The constructor is variadic:

```php
(new Worker(
    LaravelExtension::create(),
    PHPUnitExtension::create(),
    CompanyRulesExtension::create(),
))->run();
```

This shares one process pool and one PHP runtime among the extensions. It is useful when packages are always deployed together. Use separate configured hosts when extensions require different PHP binaries, environments, working directories, timeouts, or worker counts.

## Command-scoped process-local state

Rules, plugins, providers, and reducers are constructed once per PHP worker process. Their object properties persist across requests handled by that process. The processes belong to one Mago command and are shut down when that command finishes.

This state is:

- local to one worker, not shared with sibling processes;
- retained across files and incremental-analysis generations while the process lives;
- lost if Mago restarts the worker after a failure;
- not guaranteed to receive requests in file order;
- potentially observed by interleaved Fibers if callbacks cooperatively suspend.

Use generation-aware SDK data for analysis caches. Use a `WorkerReducer` when process-local application metrics must be combined at shutdown.

## PHP version

Mago sends its configured target PHP version during registration. It is exposed as `Mago\Sdk\PHPVersion` through `SourceFile` and analyzer contexts.

```php
if ($context->phpVersion->isAtLeast(PHPVersion::fromParts(8, 4))) {
    // Apply PHP 8.4-specific behavior.
}
```

The numeric identifier uses `0xMMMMmmpp`: a 16-bit major version followed by 8-bit minor and patch components. Prefer `fromParts()`, `major()`, `minor()`, `patch()`, and `isAtLeast()` over inspecting `id` directly.

## Cancellation

Every callback context includes a `CancellationTokenInterface`. Mago sends a cancellation frame when an outer request reaches its configured timeout. Closing the input stream or shutting down the worker also cancels every in-flight context locally.

```php
foreach ($largeCollection as $item) {
    $context->cancellation->throwIfCancelled();
    process($item);
}
```

`isCancelled()` performs a non-throwing check. `subscribe()` installs a callback and returns a subscription identifier; `unsubscribe()` removes it. A zero identifier means no subscription was retained. Cancellation is cooperative: CPU-bound code must check the token itself.

## Exceptions

Do not catch `Mago\Sdk\Exception\CancelledException` unless it is immediately rethrown. Other callback exceptions become protocol errors with callback identity and the original message. Optional analyzer provider failures fall back to native analysis, while registration and lifecycle failures fail the operation.
