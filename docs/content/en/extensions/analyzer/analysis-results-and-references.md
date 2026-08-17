+++
title = "Analysis results and references"
description = "Read file and project artifacts and contribute framework-known symbol edges."
nav_order = 90
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Analysis results and references

After-file, targeted-analysis, and after-analysis hooks can inspect completed native artifacts. Extensions can also contribute references that Mago could not observe directly, such as framework dispatch from a router to a controller method.

## File analysis

`FileAnalysis` exposes cheap summary fields immediately:

| Member | Meaning |
| :--- | :--- |
| `file` | Logical path used by Mago |
| `size` | Source size in bytes |
| `expressionCount` | Number of inferred expression entries |
| `inferredReturnCount` | Number of inferred return types |
| `inferredYieldKeyCount` | Number of inferred generator key types |
| `inferredYieldValueCount` | Number of inferred generator value types |
| `references` | Body-edge and signature-edge counts, plus total key entries across Mago's reference maps |

More expensive artifacts are lazy:

```php
$type = $analysis->getExpressionType($nodeOrSpan);

$types = $analysis->getMultipleExpressionTypes([
    $firstSpan,
    $secondSpan,
]);

$source = $analysis->getSourceFile();
$all = $analysis->getAllExpressionTypes();
$returns = $analysis->getInferredReturnTypes();
$yieldKeys = $analysis->getInferredYieldKeyTypes();
$yieldValues = $analysis->getInferredYieldValueTypes();
```

`getExpressionType()` returns `null` when Mago has no type for the exact span. Its plural form preserves order and batches missing values. `getAllExpressionTypes()` returns `ExpressionType` pairs of span and type.

`getSourceFile()` returns the exact in-memory bytes and syntax Mago analyzed. It never rereads the filesystem, which is important in editor and watch workflows.

If `ExpressionTypes` was requested by an after-file hook, direct span lookups are decoded from embedded records. Otherwise, exact-span lookups are fetched lazily and cached, including missing results and prefetched neighboring entries. `getSourceFile()` and inferred return or yield lists are cached after their first lazy request. `getAllExpressionTypes()` performs a fresh nested request on every call.

## Project analysis

`AfterAnalysisContext::$analysis` is a `ProjectAnalysis` containing:

- `files`, the final list of `FileAnalysis` values;
- `issueCount`, the number of issues merged before any after-analysis callback issues are added;
- `references`, the final `SymbolReferences` graph.

`getFile()` and `getMultipleFiles()` index file analyses by logical path. The plural form returns `null` for missing paths and preserves input order.

## Contributing references

Mago's unused-symbol checks rely on its reference graph. A framework plugin should add edges for calls, reads, writes, or overrides performed by framework machinery:

```php
use Acme\Framework\Http\UserController;
use Acme\Framework\Hydration\Hydrator;
use Acme\Model\User;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\ReferenceOrigin;

$controller = new MemberIdentifier(UserController::class, 'show');

$context->references->add(
    ReferenceOrigin::file('routes/web.php'),
    $controller,
);

$context->references->addPropertyWrite(
    Hydrator::class,
    new MemberIdentifier(User::class, 'email'),
);
```

A source can be a global symbol name, a `MemberIdentifier`, or `ReferenceOrigin::file()`. A target is a global symbol or member. The default kind is `Body`.

Convenience methods cover property reads, property writes, overridden members, and function-like return references. The complete `ReferenceKind` set is:

- `Body`;
- `Signature`;
- `OverriddenMember`;
- `FunctionLikeReturn`;
- `PropertyRead`;
- `PropertyWrite`.

Before-analysis references are project-wide. References added by after-file and targeted hooks are scoped to that file, so Mago can replace them correctly during incremental reanalysis.

## Reading the final graph

`SymbolReferences` is a lazy, read-only view of the final merged graph. Summary counts are available as `body`, `signature`, and `maps`, with the same values grouped in the `summary` object. `maps` counts key entries across Mago's internal reference maps; it is not a count of graph edges.

```php
$incoming = $context->analysis->references->getReferencesTo($controller);
$outgoing = $context->analysis->references->getReferencesFrom($controller);
```

Use `getMultipleReferencesTo()` and `getMultipleReferencesFrom()` for several symbols. Results are grouped in input order and cached. A `SymbolReference` exposes its `source`, `target`, and semantic `kind`.

The final graph includes native references and accepted extension contributions. It is available after merging, so it belongs in an after-analysis hook rather than a per-file provider.

## Common patterns

### Framework entry points

When a framework invokes a known family of methods, prefer registry entry-point declarations. They are matched entirely in Mago and require no callback per method. Add references manually when the relationship depends on framework data discovered by the extension.

### Unused checks

Query incoming references in batches during the after-analysis hook, then report declarations with no accepted callers. Account for reference kinds: a signature-only edge may not mean a callable is executed.

### Coverage metrics

Request expression types in an after-file hook, collect compact counters in each worker, and combine them with a [worker reducer](/extensions/sdk/worker-reduction/) if an external service needs one project-wide payload.
