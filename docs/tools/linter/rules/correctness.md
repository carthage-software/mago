---
title: Correctness Rules
---

# Correctness Rules

This document details the rules available in the `Correctness` category.

## Available Rules

| Rule | Code |
| :--- | :---------- |
| Assert Description | [`assert-description`](#assert-description) |
| Identity Comparison | [`identity-comparison`](#identity-comparison) |
| Invalid Open Tag | [`invalid-open-tag`](#invalid-open-tag) |
| No Assign In Condition | [`no-assign-in-condition`](#no-assign-in-condition) |
| No Boolean Literal Comparison | [`no-boolean-literal-comparison`](#no-boolean-literal-comparison) |
| No Empty Catch Clause | [`no-empty-catch-clause`](#no-empty-catch-clause) |
| Parameter Type | [`parameter-type`](#parameter-type) |
| Property Type | [`property-type`](#property-type) |
| Return Type | [`return-type`](#return-type) |
| Strict Assertions | [`strict-assertions`](#strict-assertions) |
| Strict Behavior | [`strict-behavior`](#strict-behavior) |
| Strict Types | [`strict-types`](#strict-types) |

---

## <a id="assert-description"></a>`assert-description`

Detects assert functions that do not have a description.

Assert functions should have a description to make it easier to understand the purpose of the assertion.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct Code

```php
<?php

assert($user->isActivated(), 'User MUST be activated at this point.');
```

#### Incorrect Code

```php
<?php

assert($user->isActivated());
```

---

## <a id="identity-comparison"></a>`identity-comparison`

Detects equality and inequality comparisons that should use identity comparison operators.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct Code

```php
<?php

if ($a === $b) {
    echo '$a is same as $b';
}
```

#### Incorrect Code

```php
<?php

if ($a == $b) {
    echo '$a is same as $b';
}
```

---

## <a id="invalid-open-tag"></a>`invalid-open-tag`

Detects misspelled PHP opening tags like `<php?` instead of `<?php`.

A misspelled opening tag will cause the PHP interpreter to treat the
following code as plain text, leading to the code being output directly
to the browser instead of being executed. This can cause unexpected
behavior and potential security vulnerabilities.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct Code

```php
<?php

echo 'Hello, world!';
```

#### Incorrect Code

```php
<php?

echo 'Hello, world!';
```

---

## <a id="no-assign-in-condition"></a>`no-assign-in-condition`

Detects assignments in conditions which can lead to unexpected behavior and make the code harder
to read and understand.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct Code

```php
<?php

$x = 1;
if ($x == 1) {
    // ...
}
```

#### Incorrect Code

```php
<?php

if ($x = 1) {
    // ...
}
```

---

## <a id="no-boolean-literal-comparison"></a>`no-boolean-literal-comparison`

Disallows comparisons where a boolean literal is used as an operand.

Comparing with a boolean literal (`true` or `false`) is redundant and can often be simplified.
For example, `if ($x === true)` is equivalent to the more concise `if ($x)`, and
`if ($y !== false)` is the same as `if ($y)`.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct Code

```php
<?php

if ($x) { /* ... */ }
if (!$y) { /* ... */ }
```

#### Incorrect Code

```php
<?php

if ($x === true) { /* ... */ }
if ($y != false) { /* ... */ }
```

---

## <a id="no-empty-catch-clause"></a>`no-empty-catch-clause`

Warns when a `catch` clause is empty.

An empty `catch` clause suppresses exceptions without handling or logging them,
potentially hiding errors that should be addressed. This practice, known as
"exception swallowing," can make debugging significantly more difficult.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct Code

```php
<?php

try {
    // some code that might throw an exception
} catch(Exception $e) {
    // Handle the error, log it, or re-throw it.
    error_log($e->getMessage());
}
```

#### Incorrect Code

```php
<?php

try {
    // some code
} catch(Exception $e) {
    // This block is empty and swallows the exception.
}
```

---

## <a id="parameter-type"></a>`parameter-type`

Detects parameters that are missing a type hint.


### Requirements

- **PHP Version:** PHP `>= 7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `ignore-closure` | `boolean` | `false` |
| `ignore-arrow-function` | `boolean` | `false` |

### Examples

#### Correct Code

```php
<?php

function foo(string $bar): void
{
    // ...
}
```

#### Incorrect Code

```php
<?php

function foo($bar): void
{
    // ...
}
```

---

## <a id="property-type"></a>`property-type`

Detects class-like properties that are missing a type hint.


### Requirements

- **PHP Version:** PHP `>= 7.4.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct Code

```php
<?php

class Foo
{
    public int $bar;
}
```

#### Incorrect Code

```php
<?php

class Foo
{
    public $bar;
}
```

---

## <a id="return-type"></a>`return-type`

Detects functions, methods, closures, and arrow functions that are missing a return type hint.


### Requirements

- **PHP Version:** PHP `>= 7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `ignore-closure` | `boolean` | `false` |
| `ignore-arrow-function` | `boolean` | `false` |

### Examples

#### Correct Code

```php
<?php

function foo(): int {
    return 42;
}
```

#### Incorrect Code

```php
<?php

function foo() {
    return 42;
}
```

---

## <a id="strict-assertions"></a>`strict-assertions`

Detects non-strict assertions in test methods.
Assertions should use strict comparison methods, such as `assertSame` or `assertNotSame`
instead of `assertEquals` or `assertNotEquals`.


### Requirements

- **Integration:** `PHPUnit`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct Code

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertSame(42, 42);
    }
}
```

#### Incorrect Code

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertEquals(42, 42);
    }
}
```

---

## <a id="strict-behavior"></a>`strict-behavior`

Detects functions relying on loose comparison unless the `$strict` parameter is specified.
The use of loose comparison for these functions may lead to hard-to-debug, unexpected behaviors.


### Requirements

- **PHP Version:** PHP `>= 7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `allow-loose-behavior` | `boolean` | `false` |

### Examples

#### Correct Code

```php
<?php

in_array(1, ['foo', 'bar', 'baz'], strict: true);
```

#### Incorrect Code

```php
<?php

in_array(1, ['foo', 'bar', 'baz']);
```

---

## <a id="strict-types"></a>`strict-types`

Detects missing `declare(strict_types=1);` statement at the beginning of the file.


### Requirements

- **PHP Version:** PHP `>= 7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `allow-disabling` | `boolean` | `false` |

### Examples

#### Correct Code

```php
<?php

declare(strict_types=1);

echo "Hello, World!";
```

#### Incorrect Code

```php
<?php

echo "Hello, World!";
```

---
