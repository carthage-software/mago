---
title: Clarity rules
outline: [2, 3]
---

# Clarity rules

This document details the rules available in the `Clarity` category.

| Rule | Code |
| :--- | :---------- |
| Explicit Octal | [`explicit-octal`](#explicit-octal) |
| Literal Named Argument | [`literal-named-argument`](#literal-named-argument) |
| No Empty | [`no-empty`](#no-empty) |
| No Hash Emoji | [`no-hash-emoji`](#no-hash-emoji) |
| No Multi Assignments | [`no-multi-assignments`](#no-multi-assignments) |
| No Nested Ternary | [`no-nested-ternary`](#no-nested-ternary) |
| No Shorthand Ternary | [`no-shorthand-ternary`](#no-shorthand-ternary) |
| No Variable Variable | [`no-variable-variable`](#no-variable-variable) |
| Str Contains | [`str-contains`](#str-contains) |
| Str Starts With | [`str-starts-with`](#str-starts-with) |
| Tagged FIXME | [`tagged-fixme`](#tagged-fixme) |
| Tagged TODO | [`tagged-todo`](#tagged-todo) |
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
| `enabled` | `boolean` | `false` |
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


## <a id="str-contains"></a>`str-contains`

Detects `strpos($a, $b) !== false` comparisons and suggests replacing them with `str_contains($a, $b)`
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

