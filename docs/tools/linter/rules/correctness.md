---
title: Correctness rules
outline: [2, 3]
---

# Correctness rules

This document details the rules available in the `Correctness` category.

| Rule | Code |
| :--- | :---------- |
| Assert Description | [`assert-description`](#assert-description) |
| Constant Type | [`constant-type`](#constant-type) |
| Identity Comparison | [`identity-comparison`](#identity-comparison) |
| Ineffective Format Ignore Next | [`ineffective-format-ignore-next`](#ineffective-format-ignore-next) |
| Ineffective Format Ignore Region | [`ineffective-format-ignore-region`](#ineffective-format-ignore-region) |
| Invalid Open Tag | [`invalid-open-tag`](#invalid-open-tag) |
| No Assign In Argument | [`no-assign-in-argument`](#no-assign-in-argument) |
| No Assign In Condition | [`no-assign-in-condition`](#no-assign-in-condition) |
| No Boolean Literal Comparison | [`no-boolean-literal-comparison`](#no-boolean-literal-comparison) |
| No Empty Catch Clause | [`no-empty-catch-clause`](#no-empty-catch-clause) |
| No Only | [`no-only`](#no-only) |
| Parameter Type | [`parameter-type`](#parameter-type) |
| Property Type | [`property-type`](#property-type) |
| Return Type | [`return-type`](#return-type) |
| Strict Assertions | [`strict-assertions`](#strict-assertions) |
| Strict Behavior | [`strict-behavior`](#strict-behavior) |
| Strict Types | [`strict-types`](#strict-types) |
| Use Specific Assertions | [`use-specific-assertions`](#use-specific-assertions) |


## <a id="assert-description"></a>`assert-description`

Detects assert functions that do not have a description.

Assert functions should have a description to make it easier to understand the purpose of the assertion.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

assert($user->isActivated(), 'User MUST be activated at this point.');
```

#### Incorrect code

```php
<?php

assert($user->isActivated());
```


## <a id="constant-type"></a>`constant-type`

Detects class constants that are missing a type hint.


### Requirements

- **PHP version:** >= `8.3.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

declare(strict_types=1);

namespace Psl\IO\Internal;

use Psl\IO;

class ResourceHandle implements IO\CloseSeekReadWriteStreamHandleInterface {
    use IO\ReadHandleConvenienceMethodsTrait;
    use IO\WriteHandleConvenienceMethodsTrait;

    public const int DEFAULT_READ_BUFFER_SIZE = 4096;
    public const int MAXIMUM_READ_BUFFER_SIZE = 786432;

    // ...
}
```

#### Incorrect code

```php
<?php

declare(strict_types=1);

namespace Psl\IO\Internal;

use Psl\IO;

class ResourceHandle implements IO\CloseSeekReadWriteStreamHandleInterface {
    use IO\ReadHandleConvenienceMethodsTrait;
    use IO\WriteHandleConvenienceMethodsTrait;

    public const DEFAULT_READ_BUFFER_SIZE = 4096;
    public const MAXIMUM_READ_BUFFER_SIZE = 786432;

    // ...
}
```


## <a id="identity-comparison"></a>`identity-comparison`

Detects equality and inequality comparisons that should use identity comparison operators.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

if ($a === $b) {
    echo '$a is same as $b';
}
```

#### Incorrect code

```php
<?php

if ($a == $b) {
    echo '$a is same as $b';
}
```


## <a id="ineffective-format-ignore-next"></a>`ineffective-format-ignore-next`

Detects `@mago-format-ignore-next` markers that will have no effect.

The formatter's ignore-next marker works at the statement level. When a
marker is placed inside an expression (like function call arguments,
array elements, or other non-statement contexts), it will not affect
the formatter's output.

To effectively ignore the next statement, place the marker immediately
before a complete statement at the top level of a block or file.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// This works - marker is before a statement
// @mago-format-ignore-next
const GRID = [
  [1, 2, 3], [1, 2, ], [0,    0],
];

foo();
```

#### Incorrect code

```php
<?php

// This doesn't work - marker is inside an array literal
$arr = [ // @mago-format-ignore-next
    1,
    2,
];
```


## <a id="ineffective-format-ignore-region"></a>`ineffective-format-ignore-region`

Detects `@mago-format-ignore-start` markers that will have no effect.

The formatter's ignore regions work at the statement level. When an
ignore marker is placed inside an expression (like function call arguments,
array elements, or other non-statement contexts), it will not affect
the formatter's output.

To effectively ignore a region, place the ignore markers between complete
statements at the top level of a block or file.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// This works - markers are between statements
// @mago-format-ignore-start
$x = 1;  $y = 2;  // preserved as-is
// @mago-format-ignore-end

foo();
```

#### Incorrect code

```php
<?php

// This doesn't work - markers are inside a function call
foo( // @mago-format-ignore-start
    $x,
    $y
// @mago-format-ignore-end
);
```


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

#### Correct code

```php
<?php

echo 'Hello, world!';
```

#### Incorrect code

```php
<php?

echo 'Hello, world!';
```


## <a id="no-assign-in-argument"></a>`no-assign-in-argument`

Detects assignments in function call arguments which can lead to unexpected behavior and make
the code harder to read and understand.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$x = 5;
foo($x);
```

#### Incorrect code

```php
<?php

foo($x = 5);
```


## <a id="no-assign-in-condition"></a>`no-assign-in-condition`

Detects assignments in conditions which can lead to unexpected behavior and make the code harder
to read and understand.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$x = 1;
if ($x == 1) {
    // ...
}
```

#### Incorrect code

```php
<?php

if ($x = 1) {
    // ...
}
```


## <a id="no-boolean-literal-comparison"></a>`no-boolean-literal-comparison`

Disallows comparisons where a boolean literal is used as an operand.

Comparing with a boolean literal (`true` or `false`) is redundant and can often be simplified.
For example, `if ($x === true)` is equivalent to the more concise `if ($x)`, and
`if ($y !== false)` is the same as `if ($y)`.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

if ($x) { /* ... */ }
if (!$y) { /* ... */ }
```

#### Incorrect code

```php
<?php

if ($x === true) { /* ... */ }
if ($y != false) { /* ... */ }
```


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

#### Correct code

```php
<?php

try {
    // some code that might throw an exception
} catch(Exception $e) {
    // Handle the error, log it, or re-throw it.
    error_log($e->getMessage());
}
```

#### Incorrect code

```php
<?php

try {
    // some code
} catch(Exception $e) {
    // This block is empty and swallows the exception.
}
```


## <a id="no-only"></a>`no-only`

Detects usage of `->only()` in Pest tests which should not be committed.

The `->only()` modifier causes only that specific test to run, which can lead to
incomplete test coverage if accidentally committed to the repository.


### Requirements

- **Integration:** `Pest`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

test('example test', function () {
    expect(true)->toBeTrue();
});

it('does something', function () {
    expect(1)->toBe(1);
});
```

#### Incorrect code

```php
<?php

test('example test', function () {
    expect(true)->toBeTrue();
})->only();

it('does something', function () {
    expect(1)->toBe(1);
})->only();
```


## <a id="parameter-type"></a>`parameter-type`

Detects parameters that are missing a type hint.


### Requirements

- **PHP version:** >= `7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `ignore-closure` | `boolean` | `false` |
| `ignore-arrow-function` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

function foo(string $bar): void
{
    // ...
}
```

#### Incorrect code

```php
<?php

function foo($bar): void
{
    // ...
}
```


## <a id="property-type"></a>`property-type`

Detects class-like properties that are missing a type hint.


### Requirements

- **PHP version:** >= `7.4.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

class Foo
{
    public int $bar;
}
```

#### Incorrect code

```php
<?php

class Foo
{
    public $bar;
}
```


## <a id="return-type"></a>`return-type`

Detects functions, methods, closures, and arrow functions that are missing a return type hint.


### Requirements

- **PHP version:** >= `7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |
| `ignore-closure` | `boolean` | `false` |
| `ignore-arrow-function` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

function foo(): int {
    return 42;
}
```

#### Incorrect code

```php
<?php

function foo() {
    return 42;
}
```


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

#### Correct code

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

#### Incorrect code

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


## <a id="strict-behavior"></a>`strict-behavior`

Detects functions relying on loose comparison unless the `$strict` parameter is specified.
The use of loose comparison for these functions may lead to hard-to-debug, unexpected behaviors.


### Requirements

- **PHP version:** >= `7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `allow-loose-behavior` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

in_array(1, ['foo', 'bar', 'baz'], strict: true);
```

#### Incorrect code

```php
<?php

in_array(1, ['foo', 'bar', 'baz']);
```


## <a id="strict-types"></a>`strict-types`

Detects missing `declare(strict_types=1);` statement at the beginning of the file.


### Requirements

- **PHP version:** >= `7.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `allow-disabling` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

declare(strict_types=1);

echo "Hello, World!";
```

#### Incorrect code

```php
<?php

echo "Hello, World!";
```


## <a id="use-specific-assertions"></a>`use-specific-assertions`

Suggests using specific PHPUnit assertions instead of generic equality assertions
when comparing with `null`, `true`, or `false`.

Using specific assertions like `assertNull`, `assertTrue`, and `assertFalse`
provides clearer error messages and makes test intent more explicit.


### Requirements

- **Integration:** `PHPUnit`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertNull($value);
        $this->assertTrue($flag);
        $this->assertFalse($condition);
    }
}
```

#### Incorrect code

```php
<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertEquals(null, $value);
        $this->assertSame(true, $flag);
        $this->assertEquals(false, $condition);
    }
}
```

