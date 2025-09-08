---
title: Suppressing Issues
---

# Suppressing Issues

While it's best to fix all issues that Mago reports, there are cases where you might need to suppress them in your source code. Mago provides two pragmas for this, each with a specific purpose: `@mago-expect` and `@mago-ignore`.

Both pragmas require you to specify the exact issue you intend to suppress, using the format `[category]:[code]`.

### Categories

There are two issue categories available:

- `lint`: For issues reported by the linter.
- `analysis`: For issues reported by the static analyzer.

## Asserting an Issue (`@mago-expect`)

This pragma asserts that a **specific** issue is expected on the line immediately following the comment. It is the strictest way to suppress an issue and is the generally recommended method.

```php
// @mago-expect lint:no-shorthand-ternary
$result = $value ?: 'default';
```

If the specified issue is found, Mago suppresses it. However, if the issue is **not** found (e.g., because the code was fixed but the pragma was left behind), Mago will report a `warning` with the code `unfulfilled-expect`. This helps you keep your suppressions up-to-date and avoid leaving obsolete comments in the code.

## Ignoring an Issue (`@mago-ignore`)

This pragma also suppresses a **specific** issue on the following line or block. It is less strict than `@mago-expect`.

```php
// @mago-ignore lint:no-shorthand-ternary
$result = $value ?: 'default';
```

If the specified issue is found, Mago suppresses it. If the issue is **not** found, Mago will report a `note` level diagnostic with the code `unused-pragma`. This is a less severe notification than the warning from `@mago-expect`, simply informing you that the pragma is unused and can be removed.

## Block-level Suppression

When a pragma is placed on the line before a block (like a function, class, or `if` statement), it will suppress the specified issue for the entire block.

```php
// @mago-ignore analysis:some-type-issue
function my_legacy_function()
{
    // The 'some-type-issue' will be ignored anywhere inside this function.
}
```

## Choosing Between `@mago-expect` and `@mago-ignore`

- Use `@mago-expect` when you want to be explicitly notified with a warning if the code changes and the issue no longer exists. This is best for most cases, as it prevents suppressions from becoming outdated.
- Use `@mago-ignore` for less critical issues or when you are less concerned about the suppression becoming obsolete. The gentle `note` will still inform you of unused pragmas without creating noise during builds or CI runs.
