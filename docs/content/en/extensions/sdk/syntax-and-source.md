+++
title = "Syntax nodes and source files"
description = "Traverse Mago syntax snapshots without reparsing PHP."
nav_order = 20
nav_section = "Extensions"
nav_subsection = "PHP SDK"
+++
# Syntax nodes and source files

Mago parses PHP in Rust and sends extensions an immutable concrete syntax tree. Extensions should use this view instead of invoking another PHP parser.

## Source files

`Mago\Sdk\Syntax\SourceFile` exposes:

| Property | Description |
| :--- | :--- |
| `phpVersion` | PHP language version selected by Mago. |
| `path` | Logical source name supplied by Mago. It may be relative, absolute, or synthetic for an in-memory source. |
| `contents` | Source bytes retained for this snapshot. Complete snapshots contain the exact in-memory bytes parsed by Mago; a targeted snapshot without `SourceText` contains an empty string. |

Spans always use byte offsets from Mago's original in-memory source, not Unicode character positions or line/column pairs. Read text from `contents` only when the callback contract includes source bytes.

## Nodes and spans

Each `Node` has a stable identifier within its snapshot, a `NodeKind`, a half-open `Span`, and an optional parent identifier.

```php
$text = $source->getText($node);
$parent = $source->getParent($node);
$children = $source->getChildren($node);
$ancestors = $source->getAncestors($node);
$calls = $source->getDescendants($node, NodeKind::FunctionCall);
```

`Span::$start` is inclusive and `Span::$end` is exclusive. `length()` returns their difference. `contains()` tests full range containment.

The traversal methods operate only on nodes included in the current snapshot. Codebase-scan snapshots and `FileAnalysis::getSourceFile()` contain complete syntax. Linter and targeted-analysis snapshots are deliberately filtered to active targets and requested structural data; do not assume `getNodes()` represents every node in the original file.

## Target nodes

`getTargetNodes()` returns the nodes Mago selected for the current callback family. For a linter request, these are nodes matching at least one active rule. Targeted analyzer contexts already expose their current `node` directly.

`getNodes(?NodeKind $kind)` returns every available node in the snapshot, optionally filtered by kind. Use targeted access where possible; broad PHP-side scans defeat native filtering.

## Text and names

`getText(Node|Span $selection)` returns the exact retained bytes for a range. If a targeted snapshot omitted `SourceText`, any non-empty original span lies outside its empty `contents` and `getText()` throws `InvalidArgumentException`. `getResolvedName()` looks up the semantic name whose span starts at the selection's start offset:

```php
$resolved = $source->getResolvedName($callNode);
if ($resolved !== null) {
    $resolved->name;     // e.g. Psl\Iter\any
    $resolved->span;     // range of the name in source
    $resolved->imported; // whether an import participated
}
```

Use resolved names instead of manually interpreting namespaces and imports. `getResolvedNames($within)` returns all available names, optionally restricted to a node or span.

## Literal strings

`getLiteralString(Node $node)` returns the decoded value of an included literal-string node when the protocol supplied a literal table. Codebase-scan snapshots include this table. Linter snapshots, targeted-analysis snapshots, and lazy complete `FileAnalysis` snapshots currently do not, so this method returns `null` for them even when the node is a literal string.

## Comment trivia

Mago's native syntax tree treats comments and whitespace as trivia. Extension snapshots intentionally serialize only comment trivia; the PHP SDK has no whitespace `TriviaKind`. This keeps snapshots smaller because exact whitespace already remains in the source bytes.

`getTrivia()` therefore returns the comments retained by the current snapshot as `Trivia` values. `TriviaKind` distinguishes single-line, multi-line, hash, and docblock comments. Read the comment text through its span:

```php
foreach ($source->getTrivia() as $comment) {
    $text = $source->getText($comment->span);
}
```

When an extension needs whitespace, inspect the relevant range of `SourceFile::$contents` rather than expecting a trivia record. A targeted analyzer hook must request `FileAnalysisRequirement::SourceText` when it needs those bytes.

## Call expressions

`CallExpression` provides a structured view over function, instance-method, null-safe-method, and static-method calls:

```php
$call = CallExpression::fromNode($source, $node);

$call->isFunction();
$call->isMethod();
$call->isStaticMethod();
$call->getName($source);

foreach ($call->arguments as $argument) {
    $argument->index;
    $argument->name;
    $argument->unpacked;
    $argument->value;
}
```

`fromNode()` throws when its input is not a supported call node or the retained node is missing the expected call structure. `fromExpression()` unwraps expression and call wrappers and returns `null` when the selection is not a supported call.

For analyzer providers, prefer the semantic `Invocation` supplied by the provider context. `CallExpression` is primarily useful to syntax-driven linter and analysis hooks.

## Generated node kinds

`NodeKind` is generated from Mago's syntax model by `just regen-sdk-node-kinds`. Its string values are the wire names used by the matched Mago and SDK release. Compare enum cases rather than depending on raw strings across releases.
