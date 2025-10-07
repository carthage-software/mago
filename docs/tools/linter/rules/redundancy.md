---
title: Redundancy rules
outline: [2, 3]
---

# Redundancy rules

This document details the rules available in the `Redundancy` category.

| Rule | Code |
| :--- | :---------- |
| Constant Condition | [`constant-condition`](#constant-condition) |
| No Closing Tag | [`no-closing-tag`](#no-closing-tag) |
| No Empty Comment | [`no-empty-comment`](#no-empty-comment) |
| No Empty Loop | [`no-empty-loop`](#no-empty-loop) |
| No Noop | [`no-noop`](#no-noop) |
| No Protected in Final | [`no-protected-in-final`](#no-protected-in-final) |
| No Redundant Block | [`no-redundant-block`](#no-redundant-block) |
| No Redundant Continue | [`no-redundant-continue`](#no-redundant-continue) |
| No Redundant File | [`no-redundant-file`](#no-redundant-file) |
| No Redundant Final | [`no-redundant-final`](#no-redundant-final) |
| No Redundant Label | [`no-redundant-label`](#no-redundant-label) |
| No Redundant Math | [`no-redundant-math`](#no-redundant-math) |
| No Redundant Method Override | [`no-redundant-method-override`](#no-redundant-method-override) |
| No Redundant Nullsafe | [`no-redundant-nullsafe`](#no-redundant-nullsafe) |
| No Redundant Parentheses | [`no-redundant-parentheses`](#no-redundant-parentheses) |
| No Redundant Readonly | [`no-redundant-readonly`](#no-redundant-readonly) |
| No Redundant String Concat | [`no-redundant-string-concat`](#no-redundant-string-concat) |
| No Redundant Use | [`no-redundant-use`](#no-redundant-use) |
| No Redundant Write Visibility | [`no-redundant-write-visibility`](#no-redundant-write-visibility) |


## <a id="constant-condition"></a>`constant-condition`

Detects `if` statements where the condition is a constant that always
evaluates to `true` or `false`.

Such statements are redundant. If the condition is always `true`, the `if`
wrapper is unnecessary. If it's always `false`, the enclosed code is dead
and can be removed or refactored.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php
if ($variable > 10) {
    echo "Greater than 10";
}
```

#### Incorrect code

```php
<?php
if (true) {
    echo "This will always run";
}

if (false) {
    echo "This is dead code";
}
```


## <a id="no-closing-tag"></a>`no-closing-tag`

Detects redundant closing tags ( `?>` ) at the end of a file.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

echo "Hello, world!";
```

#### Incorrect code

```php
<?php

echo "Hello, world!";

?>
```


## <a id="no-empty-comment"></a>`no-empty-comment`

Detects empty comments in the codebase. Empty comments are not useful and should be removed
to keep the codebase clean and maintainable.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |
| `preserve-single-line-comments` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

// This is a useful comment.
# This is also a useful comment.
/**
 * This is a docblock.
 */
```

#### Incorrect code

```php
<?php

//
#
/**/
```


## <a id="no-empty-loop"></a>`no-empty-loop`

Detects loops (`for`, `foreach`, `while`, `do-while`) that have an empty body. An empty
loop body does not perform any actions and is likely a mistake or redundant code.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

foreach ($items as $item) {
    process($item);
}
```

#### Incorrect code

```php
<?php

while (should_wait()) {
    // Empty loop body
}
```


## <a id="no-noop"></a>`no-noop`

Detects redundant `noop` statements.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

echo "Hello, world!";
```

#### Incorrect code

```php
<?php

;
```


## <a id="no-protected-in-final"></a>`no-protected-in-final`

Detects `protected` items in final classes or enums.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

final class Foo {
    private string $foo;
    private(set) string $bar;
    private string $baz;

    private function fun(): void {
        // ...
    }
}
```

#### Incorrect code

```php
<?php

final class Foo {
    protected string $foo;
    protected(set) string $bar;
    protected private(set) string $baz;

    protected function fun(): void {
        // ...
    }
}
```


## <a id="no-redundant-block"></a>`no-redundant-block`

Detects redundant blocks around statements.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

echo "Hello, world!";
```

#### Incorrect code

```php
<?php

{
    echo "Hello, world!";
}
```


## <a id="no-redundant-continue"></a>`no-redundant-continue`

Detects redundant `continue` statements in loops.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

while (true) {
    echo "Hello, world!";
}
```

#### Incorrect code

```php
<?php

while (true) {
    echo "Hello, world!";
    continue; // Redundant `continue` statement
}
```


## <a id="no-redundant-file"></a>`no-redundant-file`

Detects redundant files that contain no executable code or declarations.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

declare(strict_types=1);

function foo(): void {
    return 42;
}
```

#### Incorrect code

```php
<?php

declare(strict_types=1);
// This file is redundant.
```


## <a id="no-redundant-final"></a>`no-redundant-final`

Detects redundant `final` modifiers on methods in final classes or enum methods.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

final class Foo {
    public function bar(): void {
        // ...
    }
}
```

#### Incorrect code

```php
<?php

final class Foo {
    final public function bar(): void {
        // ...
    }
}
```


## <a id="no-redundant-label"></a>`no-redundant-label`

Detects redundant `goto` labels that are declared but not used.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

goto end;
echo "Hello, world!";
end:
```

#### Incorrect code

```php
<?php

label:
echo "Hello, world!";
```


## <a id="no-redundant-math"></a>`no-redundant-math`

Detects redundant mathematical operations that can be simplified or removed.
Includes operations like multiplying by 1/-1, adding 0, modulo 1/-1, etc.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

$result = $value * 2;
$sum = 1 + $total;
$difference = $value - 1;
$remainder = $x % 2;
```

#### Incorrect code

```php
<?php

$result = $value * 1;
$sum = 0 + $total;
$difference = $value - 0;
$remainder = $x % 1;
$negative = $value * -1;
```


## <a id="no-redundant-method-override"></a>`no-redundant-method-override`

Detects methods that override a parent method but only call the parent method with the same arguments.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

class Parent
{
    public function foo(): void
    {
        // ...
    }
}

class Child extends Parent
{
    public function foo(): void
    {
        parent::foo();

        echo 'Additional logic here';
    }
}
```

#### Incorrect code

```php
<?php

class Parent
{
    public function foo(): void
    {
        // ...
    }
}

class Child extends Parent
{
    public function foo(): void
    {
        parent::foo();
    }
}
```


## <a id="no-redundant-nullsafe"></a>`no-redundant-nullsafe`

Flags the use of the nullsafe operator (`?->`) in contexts where its null-checking behavior is redundant.

This occurs in two common situations:
1. When an expression using `?->` is immediately followed by the null coalescing operator (`??`).
2. When an expression using `?->` is checked with `isset()`.

In both scenarios, the surrounding language construct (`??` or `isset()`) already handles `null` values safely,
making the `?->` operator superfluous and the code unnecessarily verbose.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

$name = $user->name ?? 'Guest';

if (isset($user->profile)) {
    // Do something with $user->profile
}
```

#### Incorrect code

```php
<?php

$name = $user?->name ?? 'Guest';

if (isset($user?->profile)) {
    // Do something with $user->profile
}
```


## <a id="no-redundant-parentheses"></a>`no-redundant-parentheses`

Detects redundant parentheses around expressions.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

$foo = 42;
```

#### Incorrect code

```php
<?php

$foo = (42);
```


## <a id="no-redundant-readonly"></a>`no-redundant-readonly`

Detects redundant readonly modifiers on properties.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

readonly class User
{
    public $name;
}
```

#### Incorrect code

```php
<?php

readonly class User
{
    public readonly $name;
}
```


## <a id="no-redundant-string-concat"></a>`no-redundant-string-concat`

Detects redundant string concatenation expressions.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

$foo = "Hello World";
```

#### Incorrect code

```php
<?php

$foo = "Hello" . " World";
```


## <a id="no-redundant-use"></a>`no-redundant-use`

Detects `use` statements that import items that are never used.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php
namespace App;

use App\Helpers\ArrayHelper;

$result = ArrayHelper::combine([]);
```

#### Incorrect code

```php
<?php
namespace App;

use App\Helpers\ArrayHelper;
use App\Helpers\StringHelper; // StringHelper is not used.

$result = ArrayHelper::combine([]);
```


## <a id="no-redundant-write-visibility"></a>`no-redundant-write-visibility`

Detects redundant write visibility modifiers on properties.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

final class User
{
    public $name;
}
```

#### Incorrect code

```php
<?php

final class User
{
    public public(set) $name;
}
```

