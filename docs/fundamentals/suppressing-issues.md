# Suppressing Issues

While it's best to fix all issues that Mago reports, there are cases where you might need to suppress them in your source code. Mago provides two pragmas for this, each with a specific purpose: `@mago-expect` and `@mago-ignore`.

Both pragmas require you to specify the issue(s) you intend to suppress, using the format `[category]:[code]`. You can also suppress multiple issues at once by providing a comma-separated list of codes: `[category]:[code1,code2,code3]`.

To suppress multiple occurrences of the same code on a single line or within the same block, you can either repeat the code, or use the `[code](N)` count shorthand to suppress up to `N` occurrences. See [Suppressing Multiple Occurrences](#suppressing-multiple-occurrences-n) below.

## Categories

There are three issue categories available:

- `lint` (alias: `linter`): For issues reported by the linter.
- `analysis` (alias: `analyzer`, `analyser`): For issues reported by the static analyzer.
- `guard`: For issues reported by the architectural guard.

## Asserting an Issue (`@mago-expect`)

This pragma asserts that a **specific** issue is expected on the line immediately following the comment. It is the strictest way to suppress an issue and is the generally recommended method.

```php
// @mago-expect lint:no-shorthand-ternary
$result = $value ?: 'default';
```

You can also suppress multiple issues on the same line by providing a comma-separated list:

```php
// @mago-expect lint:no-shorthand-ternary,unused-variable
$result = $value ?: 'default';
```

If the specified issue(s) are found, Mago suppresses them. However, if any of the issues are **not** found (e.g., because the code was fixed but the pragma was left behind), Mago will report a `warning` with the code `unfulfilled-expect`. This helps you keep your suppressions up-to-date and avoid leaving obsolete comments in the code.

## Ignoring an Issue (`@mago-ignore`)

This pragma also suppresses **specific** issue(s) on the following line or block. It is less strict than `@mago-expect`.

```php
// @mago-ignore lint:no-shorthand-ternary
$result = $value ?: 'default';
```

Like `@mago-expect`, you can suppress multiple issues:

```php
// @mago-ignore lint:no-shorthand-ternary,no-assign-in-condition
if ($result = $value ?: 'default') {
    // Do something with $result
}
```

If the specified issue(s) are found, Mago suppresses them. If any of the issues are **not** found, Mago will report a `note` level diagnostic with the code `unused-pragma`. This is a less severe notification than the warning from `@mago-expect`, simply informing you that the pragma is unused and can be removed.

## Block-level Suppression

When a pragma is placed on the line before a block (like a function, class, or `if` statement), it will suppress the specified issue(s) for the entire block.

```php
// @mago-ignore analysis:missing-return-statement
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
    }
    // No return statement here
}
```

You can also suppress multiple issues for an entire block:

```php
// @mago-ignore analysis:missing-return-statement,unreachable-code
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
        echo 'This code is unreachable';
    }

    // No return statement here
}
```

## Suppressing Multiple Occurrences (`(N)`)

When a single line, or a block covered by a scoped pragma, triggers the same issue code more than once, repeating the code becomes tedious:

```php
// @mago-expect analysis:mixed-operand,mixed-operand,mixed-operand
return $a . $b . $c;
```

Mago supports a `(N)` shorthand to express the same thing more concisely:

```php
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;
```

`N` must be a positive integer. `code(1)` is equivalent to a plain `code`, and malformed suffixes (`(0)`, `(abc)`, unbalanced parentheses) are treated as part of the code name and will fail to match.

The count can be mixed with ordinary comma-separated codes:

```php
// @mago-expect analysis:mixed-operand(2),unused-variable
```

### Unfulfilled counts

If the actual number of matching issues is **fewer** than the expected count, Mago reports an `unfulfilled-expect` warning and the auto-fix suggests reducing the count (or dropping the suffix entirely when it falls to 1) rather than deleting the whole directive, which would re-enable the issues that _did_ match:

```php
// Before the fix: 3 matches expected, only 2 happened.
// @mago-expect analysis:mixed-operand(3)
return $a . $b;

// Auto-fix reduces the count so the 2 real matches stay suppressed:
// @mago-expect analysis:mixed-operand(2)
return $a . $b;
```

### Scoped vs. line-level counts

- For **line-level** pragmas, a bare code implicitly means "suppress up to 1 occurrence". Adding `(N)` raises the cap to `N`.
- For **scoped** (block-level) pragmas, a bare code retains its traditional behavior and suppresses _every_ matching occurrence in the block. Adding `(N)` caps the block-level suppression at `N` matches, so the `N+1`-th occurrence is reported normally. This is useful when you want to make sure no more issues than expected creep in.

## Suppressing All Issues (`all`)

You can use the special `all` code to suppress all issues at once, rather than listing individual codes. This is useful when working with legacy code that triggers many diagnostics.

### Suppressing All Issues in a Category

Use `[category]:all` to suppress every issue within a specific category:

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

### Suppressing All Issues Across All Categories

Use `all` without a category prefix to suppress every issue across all categories:

```php
// @mago-ignore all
$result = eval($value) ?: 'default';
```

When placed before a block, it suppresses all issues for the entire block:

```php
/**
 * @mago-ignore all
 */
function legacy_code(): string {
    // All linter, analyzer, and guard issues are suppressed here
}
```

::: warning
Using `all` is a blunt instrument. Prefer suppressing specific codes when possible so you don't accidentally hide new, unrelated issues.
:::

## Choosing Between `@mago-expect` and `@mago-ignore`

- Use `@mago-expect` when you want to be explicitly notified with a warning if the code changes and the issue no longer exists. This is best for most cases, as it prevents suppressions from becoming outdated.
- Use `@mago-ignore` for less critical issues or when you are less concerned about the suppression becoming obsolete. The gentle `note` will still inform you of unused pragmas without creating noise during builds or CI runs.

## Examples

Here are some practical examples of using pragma suppressions:

```php
// Suppress a guard issue
// @mago-expect guard:disallowed-use
use App\Infrastructure\SomeForbiddenClass;

// Suppress a single lint issue
// @mago-expect lint:no-shorthand-ternary
$result = $condition ?: 'default';

// Suppress issues for an entire function
// @mago-expect analysis:missing-return-statement,impossible-condition
function complexFunction(): string {
    if (false) {
        return 'never reached';
    }

    // No return statement here
}

// Suppress three occurrences of the same code on the next line
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;

// Suppress all lint issues on one line
// @mago-ignore lint:all
$result = $value ?: ($x == true ? 'yes' : 'no');

// Suppress all issues across all categories for a legacy function
// @mago-ignore all
function legacyFunction(): string {
    // Everything is suppressed here
}
```
