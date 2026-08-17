+++
title = "Targeted analysis hooks"
description = "Inspect selected nodes after Mago has completed semantic analysis."
nav_order = 60
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Targeted analysis hooks

Targeted analysis hooks inspect selected syntax or semantic targets after their containing file has completed analysis. Mago performs target matching natively. A host that registers only targeted hooks receives compact batches for matching files; when the same host also registers plain after-file hooks, their batches already contain every analyzed file and targeted callbacks are skipped inside unmatched files.

Place targeted hook implementations under `src/Mago/Analyzer/Hooks/`.

All targeted hooks implement the same contract:

```php
interface TargetedAnalysisHook
{
    public function getTargets(): array;

    public function getRequirements(): array;

    public function analyze(NodeAnalysisContext $context): void;
}
```

## Hook kinds

### Syntax nodes

`NodeAnalysisHook` returns `NodeKind` targets. Use it when the rule depends on a concrete PHP construct regardless of resolved symbols:

```php
public function getTargets(): array
{
    return [NodeKind::Attribute, NodeKind::Class_];
}
```

### Resolved method calls

`MethodCallAnalysisHook` returns `MethodTarget` values. Mago resolves the receiver hierarchy and method name before matching, so the extension does not need to inspect every call:

```php
public function getTargets(): array
{
    return [MethodTarget::exact(Builder::class, 'whereRaw')];
}
```

### Descendant declarations

`ClassLikeAnalysisHook` returns `ClassLikeTarget::descendantsOf()` values. It receives a class-like declaration when the named ancestor appears in that declaration's populated transitive parent-class or parent-interface metadata. The ancestor declaration itself never matches.

```php
public function getTargets(): array
{
    return [ClassLikeTarget::descendantsOf(Command::class)];
}
```

## Request data explicitly

The hook's `FileAnalysisRequirement` list controls which expensive artifacts Mago embeds. Mago unions the requirements of hooks that can share a target, so a context may contain data requested by another matching hook:

```php
public function getRequirements(): array
{
    return [
        FileAnalysisRequirement::TargetExpressionTypes,
        FileAnalysisRequirement::ReceiverType,
        FileAnalysisRequirement::ArgumentTypes,
        FileAnalysisRequirement::TargetSubtree,
    ];
}
```

| Requirement | `NodeAnalysisContext` effect |
| :--- | :--- |
| `TargetExpressionTypes` | Populates `targetType` when Mago inferred one |
| `ReceiverType` | Populates `receiverType` for direct method and static calls |
| `ArgumentTypes` | Populates `argumentTypes` in source order; individual entries may be `null` |
| `TargetSubtree` | Retains the target's concrete-syntax descendants in `source` |
| `SourceText` | Retains exact in-memory file bytes and all comment trivia in `source` |
| `ExpressionTypes` | Embeds every file expression type in `analysis`; generally an after-file concern |

Without `TargetSubtree`, each selected target is retained as a standalone node without its ancestors or descendants. With `TargetSubtree`, that target's descendants are retained too. Without `SourceText`, `SourceFile::$contents` is an empty string and comment trivia is omitted. Resolved names are limited to the retained target ranges. If a hook only needs the node kind, span, and semantic type, request neither.

## Analysis context

`NodeAnalysisContext` contains:

| Member | Description |
| :--- | :--- |
| `source` | Filtered `SourceFile` snapshot containing the selected node |
| `node` | Current target node |
| `analysis` | Completed `FileAnalysis` for the containing file |
| `references` | Shared file-scoped `ReferenceRegistry` |
| `targetType` | Requested target expression type, or `null` |
| `receiverType` | Requested direct receiver type, or `null` |
| `argumentTypes` | Requested direct argument types, otherwise an empty list |
| lifecycle members | PHP version, codebase, type comparator, cancellation, and `report()` |

Within one plugin callback, the plugin's after-file hooks run first and its targeted hooks then share that same file-scoped reference registry. Different plugins receive separate registries; Mago merges every plugin's contributions with the file result and replaces them if that file is reanalyzed.

## Example

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer\Hooks;

use Acme\Framework\Database\Builder;
use Mago\Sdk\Analyzer\FileAnalysisRequirement;
use Mago\Sdk\Analyzer\MethodCallAnalysisHook;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\NodeAnalysisContext;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;

final class UnsafeQueryHook implements MethodCallAnalysisHook
{
    public function getTargets(): array
    {
        return [MethodTarget::exact(Builder::class, 'whereRaw')];
    }

    public function getRequirements(): array
    {
        return [FileAnalysisRequirement::ArgumentTypes];
    }

    public function analyze(NodeAnalysisContext $context): void
    {
        $sql = $context->argumentTypes[0] ?? null;
        if ($sql?->getLiteralString() !== null) {
            return;
        }

        $context->report(
            Level::Warning,
            'non-literal-query',
            Issue::new('Use a literal query or a parameterized builder API.', $context->node->span),
        );
    }
}
```

## Avoid duplicate work

- Prefer `MethodCallAnalysisHook` over a broad call-node hook when symbol resolution determines applicability.
- Prefer `ClassLikeAnalysisHook` over scanning every class and querying ancestry in PHP.
- Request the smallest artifact set.
- Do not traverse descendants unless `TargetSubtree` was requested and the check requires them.
- Batch metadata, expression-type, and comparison lookups.
- Return quickly for the common non-match even after native targeting.
