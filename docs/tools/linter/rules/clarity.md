---
title: Clarity rules
outline: [2, 3]
---

# Clarity rules

This document details the rules available in the `Clarity` category.

| Rule | Code |
| :--- | :---------- |
| Explicit Octal | [`explicit-octal`](#explicit-octal) |
| Instanceof Stringable | [`instanceof-stringable`](#instanceof-stringable) |
| Literal Named Argument | [`literal-named-argument`](#literal-named-argument) |
| No Empty | [`no-empty`](#no-empty) |
| No Hash Emoji | [`no-hash-emoji`](#no-hash-emoji) |
| No Isset | [`no-isset`](#no-isset) |
| No Multi Assignments | [`no-multi-assignments`](#no-multi-assignments) |
| No Nested Ternary | [`no-nested-ternary`](#no-nested-ternary) |
| No Shorthand Ternary | [`no-shorthand-ternary`](#no-shorthand-ternary) |
| No Variable Variable | [`no-variable-variable`](#no-variable-variable) |
| Readable Literal | [`readable-literal`](#readable-literal) |
| Str Contains | [`str-contains`](#str-contains) |
| Str Starts With | [`str-starts-with`](#str-starts-with) |
| Tagged FIXME | [`tagged-fixme`](#tagged-fixme) |
| Tagged TODO | [`tagged-todo`](#tagged-todo) |
| Use Dedicated Expectation | [`use-dedicated-expectation`](#use-dedicated-expectation) |
| Use Simpler Expectation | [`use-simpler-expectation`](#use-simpler-expectation) |
| Use Specific Expectations | [`use-specific-expectations`](#use-specific-expectations) |
| Valid Docblock | [`valid-docblock`](#valid-docblock) |


## <a id="explicit-octal"></a>`explicit-octal`

Detects implicit octal numeral notation and suggests replacing it with explicit octal numeral notation.


### Requirements

- **PHP version:** >= `8.1.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$a = 0o123;
```

#### Incorrect code

```php
<?php

$a = 0123;
```


## <a id="instanceof-stringable"></a>`instanceof-stringable`

Detects the legacy pattern `is_object($x) && method_exists($x, '__toString')` and suggests
replacing it with `$x instanceof Stringable` for improved readability and performance.

Since PHP 8.0, all classes with `__toString()` automatically implement the `Stringable` interface.


### Requirements

- **PHP version:** >= `8.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

function stringify(mixed $value): string {
    if ($value instanceof Stringable) {
        return (string) $value;
    }

    return '';
}
```

#### Incorrect code

```php
<?php

function stringify(mixed $value): string {
    if (is_object($value) && method_exists($value, '__toString')) {
        return (string) $value;
    }

    return '';
}
```


## <a id="literal-named-argument"></a>`literal-named-argument`

Enforces that literal values used as arguments in function or method calls
are passed as **named arguments**.

This improves readability by clarifying the purpose of the literal value at the call site.
It is particularly helpful for boolean flags, numeric constants, and `null` values
where the intent is often ambiguous without the parameter name.


### Requirements

- **PHP version:** >= `8.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `check-first-argument` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

function set_option(string $key, bool $enable_feature) {}

set_option(key: 'feature_x', enable_feature: true); // ✅ clear intent
```

#### Incorrect code

```php
<?php

function set_option(string $key, bool $enable_feature) {}

set_option('feature_x', true); // ❌ intent unclear
```


## <a id="no-empty"></a>`no-empty`

Detects the use of the `empty()` construct.

The `empty()` language construct can lead to ambiguous and potentially buggy code due to
loose and counterintuitive definition of emptiness. It fails to clearly convey
developer's intent or expectation, making it preferable to use explicit checks.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

if ($myArray === []) {
    // ...
}
```

#### Incorrect code

```php
<?php

if (!empty($myArray)) {
    // ...
}
```


## <a id="no-hash-emoji"></a>`no-hash-emoji`

Discourages usage of the `#️⃣` emoji in place of the ASCII `#`.

While PHP allows the use of emojis in comments, it is generally discouraged to use them in place
of the normal ASCII `#` symbol. This is because it can confuse readers and may break external
tools that expect the normal ASCII `#` symbol.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

# This is a comment

#[MyAttribute]
class Foo {}
```

#### Incorrect code

```php
<?php

#️⃣ This is a comment

#️⃣[MyAttribute] <- not a valid attribute
class Foo {}
```


## <a id="no-isset"></a>`no-isset`

Detects the use of the `isset()` construct.

The `isset()` language construct checks whether a variable is set and is not null.
However, it can lead to ambiguous code because it conflates two distinct checks:
variable existence and null comparison. Using explicit null checks or the null
coalescing operator (`??`) is often clearer and more maintainable.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

if ($value !== null) {
    // ...
}

$result = $value ?? 'default';
```

#### Incorrect code

```php
<?php

if (isset($value)) {
    // ...
}
```


## <a id="no-multi-assignments"></a>`no-multi-assignments`

Flags any instances of multiple assignments in a single statement. This can lead to
confusion and unexpected behavior, and is generally considered poor practice.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$b = 0;
$a = $b;
```

#### Incorrect code

```php
<?php

$a = $b = 0;
```


## <a id="no-nested-ternary"></a>`no-nested-ternary`

Nested ternary expressions are disallowed to improve code clarity and prevent potential bugs arising from confusion over operator associativity.

In PHP 8.0 and later, the ternary operator (`? :`) is non-associative. Before PHP 8.0, it was left-associative, which is now deprecated. Most other programming languages treat it as right-associative. This inconsistency across versions and languages can make nested ternaries hard to reason about, even when using parentheses.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

if ($user->isAdmin()) {
    $allowed = true;
} else {
    $allowed = $user->isEditor();
}
```

#### Incorrect code

```php
<?php

$allowed = $user->isAdmin() ? true : ($user->isEditor() ? true : false);
```


## <a id="no-shorthand-ternary"></a>`no-shorthand-ternary`

Detects the use of the shorthand ternary and elvis operators.

Both shorthand ternary operator (`$a ? : $b`) and elvis operator (`$a ?: $b`) relies on loose comparison.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$value = $foo ?? $default;
$value = $foo ? $foo : $default;
```

#### Incorrect code

```php
<?php
$value = $foo ?: $default;
$value = $foo ? : $default;
```


## <a id="no-variable-variable"></a>`no-variable-variable`

Discourages usage of PHP's variable variables feature.

Variable variables can make code harder to read and maintain, as they introduce a level of indirection that can confuse readers and complicate static analysis.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$foo = 'bar';

echo $foo; // Outputs 'bar'
```

#### Incorrect code

```php
<?php

$foo = 'bar';
$varName = 'foo';

echo $$varName; // Outputs 'bar'
```


## <a id="readable-literal"></a>`readable-literal`

Enforces using underscore separators in numeric literals for improved readability.


### Requirements

- **PHP version:** >= `7.4.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `min-digits` | `integer` | `5` |

### Examples

#### Correct code

```php
<?php

$a = 1_000_000;
$b = 0xCAFE_F00D;
$c = 0b0101_1111;
```

#### Incorrect code

```php
<?php

$a = 1000000;
$b = 0xCAFEF00D;
$c = 0b01011111;
```


## <a id="str-contains"></a>`str-contains`

Detects `strpos($a, $b) !== false` and `strpos($a, $b) === false` comparisons and suggests
replacing them with `str_contains($a, $b)` or `!str_contains($a, $b)` for improved readability
and intent clarity.


### Requirements

- **PHP version:** >= `8.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$a = 'hello world';
$b = 'world';

if (str_contains($a, $b)) {
    echo 'Found';
}
```

#### Incorrect code

```php
<?php

$a = 'hello world';
$b = 'world';

if (strpos($a, $b) !== false) {
    echo 'Found';
}
```


## <a id="str-starts-with"></a>`str-starts-with`

Detects `strpos($a, $b) === 0` comparisons and suggests replacing them with `str_starts_with($a, $b)`
for improved readability and intent clarity.


### Requirements

- **PHP version:** >= `8.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$a = 'hello world';
$b = 'hello';
if (str_starts_with($a, $b)) {
    echo 'Found';
}
```

#### Incorrect code

```php
<?php

$a = 'hello world';
$b = 'hello';
if (strpos($a, $b) === 0) {
    echo 'Found';
}
```


## <a id="tagged-fixme"></a>`tagged-fixme`

Detects FIXME comments that are not tagged with a user or issue reference. Untagged FIXME comments
are not actionable and can be easily missed by the team. Tagging the FIXME comment with a user or
issue reference ensures that the issue is tracked and resolved.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// FIXME(@azjezz) This is a valid FIXME comment.
// FIXME(azjezz) This is a valid FIXME comment.
// FIXME(#123) This is a valid FIXME comment.
```

#### Incorrect code

```php
<?php

// FIXME: This is an invalid FIXME comment.
```


## <a id="tagged-todo"></a>`tagged-todo`

Detects TODO comments that are not tagged with a user or issue reference. Untagged TODOs
can be difficult to track and may be forgotten. Tagging TODOs with a user or issue reference
makes it easier to track progress and ensures that tasks are not forgotten.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// TODO(@azjezz) This is a valid TODO comment.
// TODO(azjezz) This is a valid TODO comment.
// TODO(#123) This is a valid TODO comment.
```

#### Incorrect code

```php
<?php

// TODO: This is an invalid TODO comment.
```


## <a id="use-dedicated-expectation"></a>`use-dedicated-expectation`

Use dedicated matchers instead of function calls in Pest tests.

Instead of `expect(is_array($x))->toBeTrue()`, use `expect($x)->toBeArray()`.
This provides clearer intent and better error messages.

Supported patterns:
- Type checks: is_array, is_string, is_int, is_float, is_bool, is_numeric, is_callable, is_iterable, is_object, is_resource, is_scalar, is_null
- String: str_starts_with, str_ends_with, ctype_alpha, ctype_alnum
- Array: in_array, array_key_exists
- File: is_file, is_dir, is_readable, is_writable, file_exists
- Object: property_exists


### Requirements

- **Integration:** `Pest`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

test('dedicated matchers', function () {
    expect($value)->toBeArray();
    expect($value)->toBeString();
    expect($string)->toStartWith('prefix');
    expect($array)->toContain($item);
    expect($path)->toBeFile();
    expect($obj)->toHaveProperty('name');
});
```

#### Incorrect code

```php
<?php

test('function calls', function () {
    expect(is_array($value))->toBeTrue();
    expect(is_string($value))->toBeTrue();
    expect(str_starts_with($string, 'prefix'))->toBeTrue();
    expect(in_array($item, $array))->toBeTrue();
    expect(is_file($path))->toBeTrue();
    expect(property_exists($obj, 'name'))->toBeTrue();
});
```


## <a id="use-simpler-expectation"></a>`use-simpler-expectation`

Simplify expect() expressions in Pest tests by using dedicated matchers.

This rule detects patterns where the expect() argument contains an expression that can be simplified:
- `expect(!$x)->toBeTrue()` -> `expect($x)->toBeFalse()`
- `expect(!$x)->toBeFalse()` -> `expect($x)->toBeTrue()`
- `expect($a > $b)->toBeTrue()` -> `expect($a)->toBeGreaterThan($b)`
- `expect($a >= $b)->toBeTrue()` -> `expect($a)->toBeGreaterThanOrEqual($b)`
- `expect($a < $b)->toBeTrue()` -> `expect($a)->toBeLessThan($b)`
- `expect($a <= $b)->toBeTrue()` -> `expect($a)->toBeLessThanOrEqual($b)`
- `expect($a === $b)->toBeTrue()` -> `expect($a)->toBe($b)`
- `expect($a !== $b)->toBeTrue()` -> `expect($a)->not->toBe($b)`
- `expect($x instanceof Y)->toBeTrue()` -> `expect($x)->toBeInstanceOf(Y::class)`
- `expect($x >= min && $x <= max)->toBeTrue()` -> `expect($x)->toBeBetween(min, max)`

Using dedicated matchers provides clearer intent and better error messages.


### Requirements

- **Integration:** `Pest`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

test('simplified expectations', function () {
    expect($condition)->toBeFalse();
    expect($a)->toBeGreaterThan($b);
    expect($a)->toBe($b);
    expect($obj)->toBeInstanceOf(ClassName::class);
    expect($x)->toBeBetween(1, 10);
});
```

#### Incorrect code

```php
<?php

test('complex expectations', function () {
    expect(!$condition)->toBeTrue();
    expect($a > $b)->toBeTrue();
    expect($a === $b)->toBeTrue();
    expect($obj instanceof ClassName)->toBeTrue();
    expect($x >= 1 && $x <= 10)->toBeTrue();
});
```


## <a id="use-specific-expectations"></a>`use-specific-expectations`

Use dedicated matchers instead of generic comparisons in Pest tests.

This rule suggests more specific matchers for common patterns:
- `toBe(true)` / `toEqual(true)` -> `toBeTrue()`
- `toBe(false)` / `toEqual(false)` -> `toBeFalse()`
- `toBe(null)` / `toEqual(null)` -> `toBeNull()`
- `toBe([])` / `toBe('')` -> `toBeEmpty()`
- `not->toBeFalse()` -> `toBeTrue()`
- `not->toBeTrue()` -> `toBeFalse()`

Using dedicated matchers provides clearer intent and better error messages.


### Requirements

- **Integration:** `Pest`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

test('specific matchers', function () {
    expect($value)->toBeTrue();
    expect($value)->toBeFalse();
    expect($value)->toBeNull();
    expect($array)->toBeEmpty();
});
```

#### Incorrect code

```php
<?php

test('generic comparisons', function () {
    expect($value)->toBe(true);
    expect($value)->toBe(false);
    expect($value)->toBe(null);
    expect($array)->toBe([]);
    expect($value)->not->toBeFalse();
});
```


## <a id="valid-docblock"></a>`valid-docblock`

Checks for syntax errors in docblock comments. This rule is disabled by default because
it can be noisy and may not be relevant to all codebases.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

/**
 * For more information, {@see https://example.com}.
 *
 * @param int $a
 *
 * @return int
 */
function foo($a) {
    return $a;
}
```

#### Incorrect code

```php
<?php

/**
 * For more information, {@see https://example.com
 *
 * @param int $a
 *
 * @return int
 */
function foo($a) {
    return $a;
}
```

