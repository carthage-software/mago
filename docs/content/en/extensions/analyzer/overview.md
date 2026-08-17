+++
title = "Analyzer plugins"
description = "Extend Mago's semantic understanding with targeted providers and lifecycle hooks."
nav_order = 10
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Analyzer plugins

Analyzer plugins teach Mago about behavior that cannot be derived from PHP declarations alone. A framework plugin can describe dynamic return types, magic properties, assertions, lifecycle initialization, entry points, and framework-owned references. It can also inspect completed analysis results and report its own issues.

The native analyzer remains responsible for parsing, name resolution, type inference, control flow, and diagnostics. An extension supplies narrowly targeted semantic facts through the worker protocol; it does not run a second analyzer in the extension process. This chapter expresses those contracts through the bundled PHP SDK.

## Define a plugin

Every plugin implements `Mago\Sdk\Analyzer\Plugin`. Create `src/Mago/Analyzer/AcmePlugin.php`:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer;

use Mago\Sdk\Analyzer\MethodReturnTypeProvider;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;

final class AcmePlugin implements Plugin
{
    public function __construct(
        private readonly MethodReturnTypeProvider $containerProvider,
    ) {}

    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            identifier: 'acme/framework',
            name: 'Acme framework',
            description: 'Understands Acme framework conventions.',
            aliases: ['acme'],
            defaultEnabled: true,
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerMethodReturnTypeProvider($this->containerProvider);
    }
}
```

The identifier is the stable, globally unique configuration name. Aliases provide shorter alternative names. Identifiers and aliases are compared case-insensitively and must not overlap within the plugin.

`defaultEnabled` controls analyzer activation, not whether the worker process starts. Mago still starts an enabled extension host so that it can register linter rules, other plugins, or reducers.

## Activation

Default-enabled plugins run unless `[analyzer].disable-default-plugins` is true. Explicit plugin names in `[analyzer].plugins` enable matching identifiers or aliases:

```toml
[analyzer]
disable-default-plugins = true
plugins = ["acme/framework"]
```

Use `mago extension list` to inspect configured hosts, logical extensions, and linter rules. The command does not currently print analyzer plugins; run `mago analyze` to exercise analyzer registration and plugin selection.

## Registration surface

`PluginRegistry` accepts four kinds of capability:

| Category | Capabilities |
| :--- | :--- |
| Initialization and lifecycle | Initialization, codebase scan, before-analysis, after-file, and after-analysis hooks |
| Semantic providers | Function and method return types, callable signatures, assertions, property types, property initialization, and class initializers |
| Targeted analysis | Syntax-node, resolved method-call, and descendant class-like hooks |
| Native declarations | Method entry points, attributed entry points, and issue filters |

Register each object once. One object may implement several compatible interfaces, but it must still be registered for every capability Mago should dispatch.

## Providers and hooks

A **provider** answers a semantic question while native analysis is in progress. Providers should be fast, deterministic where possible, and return `null` when they do not know the answer. Mago then tries the next provider or preserves native behavior.

A **hook** observes a lifecycle stage or completed analysis artifact. Hooks can report issues and add symbol references. Targeted hooks run only for nodes Mago matched natively.

Prefer a provider over an after-the-fact issue filter. Correct type information prevents downstream false positives and improves every later analysis step.

## Targeting

Providers and targeted hooks declare non-empty target lists. Mago performs matching in Rust before crossing the process boundary:

- `FunctionTarget`: one exact function, a prefix, or a namespace.
- `MethodTarget`: a class and method pattern; exact classes include descendants and implementations.
- `PropertyTarget`: a class and property pattern; property names omit the leading `$`.
- `ClassTarget`: one exact class hierarchy, a terminal-prefix pattern, or every class.
- `ClassLikeTarget`: declarations descending from one ancestor; the ancestor itself does not match.
- `NodeKind`: exact concrete-syntax node kinds.

For method, property, and class patterns, `*` is allowed only as the final character. Use the narrowest target that expresses the framework behavior.

Function, method, and class matching is ASCII case-insensitive. Property-name matching is case-sensitive and follows PHP property semantics. An exact method, property, or class target expands through inheritance; a class pattern containing `*` matches class names directly and does not add a separate ancestry expansion.

## Failure behavior

External semantic providers are optional hints. If a worker fails, times out, disconnects, or returns an invalid provider response, Mago logs the failure and falls back to native behavior instead of invalidating the current file's analysis.

Lifecycle hooks have broader side effects. Registration, initialization, scan, and lifecycle callback failures propagate as analysis-operation errors rather than being converted into ordinary analyzer issues. Extensions should use the cancellation token in expensive loops and avoid blocking work in hot provider callbacks.

## Provider memoization

Call `$registry->enableProviderMemoization()` only when every registered function and method return-type provider is deterministic and independent of:

- source locations;
- invocation order;
- mutable external state;
- process-local state that changes during the analysis generation.

Memoization includes effective callable signatures supplied by those providers. Incorrectly enabling it can reuse semantically wrong answers, so it is an opt-in contract rather than an automatic optimization.

## Next steps

- [Lifecycle hooks](/extensions/analyzer/lifecycle-hooks/) explains initialization through final project analysis.
- [Return types and callable signatures](/extensions/analyzer/return-types-and-callable-signatures/) covers dynamic calls.
- [Codebase metadata](/extensions/analyzer/codebase-metadata/) describes read-only project queries available to callbacks.
- [Types and comparisons](/extensions/analyzer/types-and-comparisons/) explains the complete type representation and native comparison service.
