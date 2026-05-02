+++
title = "Suppressing issues"
description = "How to use the @mago-expect and @mago-ignore pragmas to silence specific issues in your code."
nav_order = 30
nav_section = "Fundamentals"
+++
# Suppressing issues

Fixing the underlying problem is almost always the right answer. Sometimes it isn't: legacy code, false positives, deliberate exceptions. For those cases, Mago has two pragma comments you put in the source: `@mago-expect` and `@mago-ignore`.

Both take the form `category:code`, with three categories available:

- `lint` (alias `linter`) for linter issues.
- `analysis` (aliases `analyzer`, `analyser`) for analyzer issues.
- `guard` for architectural guard issues.

Multiple codes can be suppressed at once with a comma-separated list, and a `(N)` count shorthand handles the case where the same code fires N times on one line.

## `@mago-expect`

Asserts that a specific issue is expected on the line that follows. The strictest of the two pragmas, and the one we recommend by default.

```php
// @mago-expect lint:no-shorthand-ternary
$result = $value ?: 'default';
```

Multiple codes:

```php
// @mago-expect lint:no-shorthand-ternary,unused-variable
$result = $value ?: 'default';
```

If every expected code matches an actual issue, the issues are suppressed. If any expected code fails to match, Mago reports an `unfulfilled-expect` warning so the pragma does not silently linger after the underlying code is fixed.

## `@mago-ignore`

Suppresses the listed codes on the following line or block, but does not warn loudly when the code is no longer being triggered. Mago still reports an `unused-pragma` note so you can clean it up, but only at note level rather than warning.

```php
// @mago-ignore lint:no-shorthand-ternary
$result = $value ?: 'default';
```

Multiple codes work the same way:

```php
// @mago-ignore lint:no-shorthand-ternary,no-assign-in-condition
if ($result = $value ?: 'default') {
    // Do something with $result
}
```

## Block-level suppression

When a pragma sits on the line before a block (function, class, `if`, …), it covers the whole block.

```php
// @mago-ignore analysis:missing-return-statement
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
    }
    // No return statement here.
}
```

The same applies for multi-code lists:

```php
// @mago-ignore analysis:missing-return-statement,unreachable-code
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
        echo 'This code is unreachable';
    }
}
```

## Suppressing N occurrences

When a single line (or a block covered by a scoped pragma) trips the same code several times, repeating the code is tedious:

```php
// @mago-expect analysis:mixed-operand,mixed-operand,mixed-operand
return $a . $b . $c;
```

Use the `(N)` shorthand instead:

```php
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;
```

`N` must be a positive integer. `code(1)` is equivalent to a plain `code`. Malformed suffixes like `(0)`, `(abc)`, or unbalanced parens are treated as part of the code name and will fail to match.

The count mixes with normal comma lists:

```php
// @mago-expect analysis:mixed-operand(2),unused-variable
```

### Unfulfilled counts

If fewer issues match than expected, Mago reports `unfulfilled-expect` and the auto-fix lowers the count rather than removing the directive (which would re-enable any issues that did match):

```php
// Before: 3 matches expected, only 2 happened.
// @mago-expect analysis:mixed-operand(3)
return $a . $b;

// After auto-fix: count drops so the 2 real matches stay suppressed.
// @mago-expect analysis:mixed-operand(2)
return $a . $b;
```

### Line vs block semantics

For line-level pragmas, a bare code suppresses up to one occurrence; `(N)` raises the cap to `N`.

For block-level (scoped) pragmas, a bare code suppresses every matching occurrence in the block. Adding `(N)` caps the suppression at `N` matches, so the `N+1`-th matching issue still gets reported. Useful when you want to make sure no more issues than expected creep in.

## Suppressing every issue: `all`

The special `all` code suppresses everything at once. Use sparingly: it hides any new code that gets added later too.

Within a single category:

```php
// @mago-ignore lint:all
$result = $value ?: ($x == true ? 'yes' : 'no');

// @mago-expect analysis:all
function legacy_code(): string {
    if (rand(0, 1)) {
        return 'foo';
    }
}
```

Across every category:

```php
// @mago-ignore all
$result = eval($value) ?: 'default';
```

In a docblock above a block, this covers the whole block:

```php
/**
 * @mago-ignore all
 */
function legacy_code(): string {
    // Every linter, analyzer, and guard issue is suppressed here.
}
```

Prefer specific codes whenever you can. `all` is a blunt instrument that masks new issues you would otherwise want to see.

## Picking between expect and ignore

- `@mago-expect` is the right default. It guarantees you hear about the suppression once the underlying issue is fixed.
- `@mago-ignore` is the lighter option for less critical suppressions or when you accept that the pragma may quietly outlive the issue.

## Examples

```php
// Suppress a guard issue
// @mago-expect guard:disallowed-use
use App\Infrastructure\SomeForbiddenClass;

// Suppress one lint issue
// @mago-expect lint:no-shorthand-ternary
$result = $condition ?: 'default';

// Suppress issues for an entire function
// @mago-expect analysis:missing-return-statement,impossible-condition
function complexFunction(): string {
    if (false) {
        return 'never reached';
    }
}

// Three occurrences of one code on the next line
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;

// All lint issues on one line
// @mago-ignore lint:all
$result = $value ?: ($x == true ? 'yes' : 'no');

// Everything, for a legacy function
// @mago-ignore all
function legacyFunction(): string {
    // Everything suppressed here.
}
```
