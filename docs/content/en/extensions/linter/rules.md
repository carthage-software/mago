+++
title = "Writing linter rules"
description = "Define targets, inspect syntax, report issues, and suggest fixes."
nav_order = 20
nav_section = "Extensions"
nav_subsection = "Linter"
+++
# Writing linter rules

A rule implements `Mago\Sdk\Linter\Rule`:

```php
interface Rule
{
    public function getDefinition(): RuleDefinition;

    public function lint(LintContext $context): void;
}
```

## Rule definition

```php
public function getDefinition(): RuleDefinition
{
    return new RuleDefinition(
        code: 'acme/prefer-array-any',
        name: 'Prefer array_any',
        description: 'Suggests array_any instead of Psl\\Iter\\any.',
        defaultLevel: Level::Help,
        defaultEnabled: true,
        targets: [NodeKind::FunctionCall],
    );
}
```

| Field | Contract |
| :--- | :--- |
| `code` | Globally unique, non-empty issue code. It must not collide with a native rule or another external rule. A vendor-qualified code is recommended. |
| `name` | Non-empty human-readable rule name. |
| `description` | Non-empty concise description shown by extension inspection. |
| `defaultLevel` | `Note`, `Help`, `Warning`, or `Error`. Applied to every issue from this rule. |
| `defaultEnabled` | Whether normal lint runs activate the rule. `--only` can select it explicitly. |
| `targets` | Non-empty, duplicate-free list of exact `NodeKind` cases. |

`getDefinition()` runs during worker construction and registration. Return stable metadata; do not derive it from the current source file.

## Lint context

For each matching node, `lint()` receives:

| Member | Description |
| :--- | :--- |
| `file` | Filtered immutable `SourceFile`. |
| `node` | Current target node. |
| `cancellation` | Cooperative cancellation token. |
| `report(Issue)` | Adds an issue under this rule's code and level. |
| `getParent()` | Parent node, when available. |
| `getChildren()` | Direct children of the target. |
| `getText()` | Exact target source text. |
| `getResolvedName()` | Resolved name beginning at the target, when one exists. |

Do not retain a `LintContext` after `lint()` returns. Retaining immutable values for a process-local cache is possible, but account for memory use and changing in-memory file contents.

## Complete call rule

Place the complete rule at `src/Mago/Linter/Rules/PreferArrayAnyRule.php`:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Linter\Rules;

use Mago\Sdk\Linter\LintContext;
use Mago\Sdk\Linter\Rule;
use Mago\Sdk\Linter\RuleDefinition;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Reporting\TextEdit;
use Mago\Sdk\Syntax\NodeKind;

use function strcasecmp;

final class PreferArrayAnyRule implements Rule
{
    public function getDefinition(): RuleDefinition
    {
        return new RuleDefinition(
            code: 'acme/prefer-array-any',
            name: 'Prefer array_any',
            description: 'Suggests array_any instead of Psl\\Iter\\any.',
            defaultLevel: Level::Help,
            defaultEnabled: true,
            targets: [NodeKind::FunctionCall],
        );
    }

    public function lint(LintContext $context): void
    {
        $resolved = $context->getResolvedName();
        if ($resolved === null || strcasecmp($resolved->name, 'Psl\\Iter\\any') !== 0) {
            return;
        }

        $context->report(
            Issue::new(
                'Prefer array_any() over Psl\\Iter\\any().',
                $context->node->span,
            )
                ->withHelp('Replace this helper call with array_any().')
                ->withEdit(TextEdit::replace($resolved->span, 'array_any')),
        );
    }
}
```

The rule uses Mago's resolved name rather than comparing raw source, so imported and fully qualified calls are handled consistently. The edit replaces only the name span, preserving the original arguments and formatting.

## Choosing targets

Choose the narrowest node kinds that can trigger the rule. A rule interested in function calls should target `FunctionCall`, not `Expression` or `Program`. Narrow targets reduce snapshot size and PHP dispatch work.

Several targets are appropriate when syntax variants share one check:

```php
targets: [
    NodeKind::MethodCall,
    NodeKind::NullSafeMethodCall,
    NodeKind::StaticMethodCall,
],
```

Mago rejects duplicate targets. One syntax node may be visited by several rules, but a rule is invoked once for each of its matching target nodes.

## Traversal

Use `$context->getChildren()` when the check depends on the grammar immediately below the target. For a deeper walk, use `$context->file->getDescendants($context->node)` sparingly: the same descendant may also be delivered as its own target, and repeated subtree walks can create quadratic work.

`CallExpression::fromNode($context->file, $context->node)` provides call structure, named arguments, unpacking, receivers, and member names for a known call target. It throws for a non-call node; use `fromExpression()` when the input may not be a call. Resolved names remain the preferred way to identify functions and classes.

## Reporting and fixes

Call `report()` once per diagnostic. An issue can contain secondary annotations, notes, help, a documentation link, and multiple edits. See [Reporting issues and suggested edits](/extensions/sdk/reporting/).

Fixes should use source spans already supplied by Mago. Never search and replace the entire file when a precise node or resolved-name span is available.

## Performance checklist

- Target the narrowest possible `NodeKind` set.
- Return immediately for the common non-match.
- Use resolved names instead of reparsing namespace imports.
- Avoid scanning `SourceFile::$contents` for every target.
- Cache immutable package configuration on the rule object.
- Check cancellation in long loops.
- Do not perform blocking network I/O from a hot rule callback.
