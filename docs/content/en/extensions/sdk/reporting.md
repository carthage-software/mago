+++
title = "Reporting issues and suggested edits"
description = "Create diagnostics, annotations, and machine-applicable fixes."
nav_order = 30
nav_section = "Extensions"
nav_subsection = "PHP SDK"
+++
# Reporting issues and suggested edits

Linter rules and analyzer hooks report the same immutable `Mago\Sdk\Reporting\Issue` value. The surrounding callback supplies the issue code and severity.

## Create an issue

Use `Issue::new()` when the callback has an implicit current file. Linter rules, after-file hooks, and targeted analysis hooks have one:

```php
$issue = Issue::new(
    'Prefer the native array_any() function.',
    $call->span,
    'This call uses a compatibility wrapper.',
);
```

Before-analysis and after-analysis hooks have no implicit file. Their annotations and edits must name a source file. Use `Issue::at()` for the primary annotation:

```php
$issue = Issue::at(
    'The registered service is never requested.',
    $serviceMetadata->location,
);
```

For analyzer hooks, a named file must be known to Mago's in-memory database. Host files and extension-provided stubs satisfy this requirement even when no physical path exists.

Linter responses are intentionally confined to the file being linted. Linter annotations are always interpreted against that file, and a named-file text edit is rejected. In a linter rule, use `Issue::new()`, `withSecondaryAnnotation()`, and current-file edit factories only.

## Add context

Builder methods return a new `Issue`; they do not mutate the original:

```php
$issue = $issue
    ->withNote('The wrapper accepts every iterable, while array_any() requires an array.')
    ->withHelp('Use array_any() only after verifying the input type.')
    ->withLink('https://example.com/rules/prefer-array-any')
    ->withSecondaryAnnotation($argument->span, 'This argument must be an array.')
    ->withSecondaryLocation($definition->location, 'The service is defined here.');
```

An issue must have a non-empty message and at least one primary annotation. Notes, help, and links must be non-empty when supplied. Annotation messages may be `null` or any string, including an empty string.

## Report from a linter rule

The rule definition owns its code and default level:

```php
$context->report($issue);
```

## Report from an analyzer hook

Analyzer lifecycle and targeted analysis contexts require the level and code at the call site:

```php
$context->report(Level::Warning, 'unknown-route', $issue);
```

The analyzer code supplied to `report()` is local to the current plugin. Mago prefixes it with the plugin identifier, separated by `/`. A plugin identified as `acme/framework` reporting `unknown-route` therefore produces the effective issue code `acme/framework/unknown-route`. Do not repeat the plugin identifier in the local code.

Analyzer providers return semantic information and cannot report issues directly. Use an analysis hook when a diagnostic is required.

## Suggested edits

Attach one or more `TextEdit` values:

```php
$issue = $issue->withEdit(TextEdit::replace($resolvedName->span, 'array_any'));
```

Available factories are:

| Factory | Result |
| :--- | :--- |
| `delete(Span)` | Delete a range in the current file. |
| `deleteAt(SourceLocation)` | Delete a range in a named file. |
| `insert(int $offset, string $text)` | Insert text at a byte offset in the current file. |
| `replace(Span, string)` | Replace a range in the current file. |
| `replaceAt(SourceLocation, string)` | Replace a range in a named file. |

Edit spans refer to the original source snapshot, even when an issue contains multiple edits. Do not calculate later edit offsets from text produced by an earlier edit.

## Safety

Edits are `Safety::Safe` by default. Change the classification with `withSafety()`:

```php
$edit = TextEdit::replace($span, $replacement)
    ->withSafety(Safety::PotentiallyUnsafe);
```

- `Safe` means the transformation is intended to preserve behavior.
- `PotentiallyUnsafe` means it is usually appropriate but needs review.
- `Unsafe` means applying it can intentionally change behavior.

The user's fix flags and safety policy decide which edits Mago applies. Classification is part of the extension's contract; do not mark a speculative rewrite safe merely to make it easier to apply.

`withFile()` changes a current-file edit into a named-file edit. Prefer the `*At()` factories when a `SourceLocation` is already available. Named-file edits are supported only by analyzer lifecycle and targeted-hook responses, not linter responses.

## Received analyzer issues

Issue-filter hooks receive `ReportedIssue`, which includes the effective `Level`, optional analyzer code, message, notes, annotations, help, link, and edits. This is a read-only representation of an issue Mago already produced.
