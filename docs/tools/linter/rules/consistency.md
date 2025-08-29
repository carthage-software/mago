---
title: Consistency rules
outline: [2, 3]
---

# Consistency rules

This document details the rules available in the `Consistency` category.

| Rule | Code |
| :--- | :---------- |
| Ambiguous Function Call | [`ambiguous-function-call`](#ambiguous-function-call) |
| Array Style | [`array-style`](#array-style) |
| Assertion Style | [`assertion-style`](#assertion-style) |
| Block Statement | [`block-statement`](#block-statement) |
| Braced String Interpolation | [`braced-string-interpolation`](#braced-string-interpolation) |
| Class Name | [`class-name`](#class-name) |
| Constant Name | [`constant-name`](#constant-name) |
| Enum Name | [`enum-name`](#enum-name) |
| Function Name | [`function-name`](#function-name) |
| Interface Name | [`interface-name`](#interface-name) |
| Lowercase Keyword | [`lowercase-keyword`](#lowercase-keyword) |
| Lowercase Type Hint | [`lowercase-type-hint`](#lowercase-type-hint) |
| No Alias Function | [`no-alias-function`](#no-alias-function) |
| No Hash Comment | [`no-hash-comment`](#no-hash-comment) |
| No Php Tag Terminator | [`no-php-tag-terminator`](#no-php-tag-terminator) |
| No Trailing Space | [`no-trailing-space`](#no-trailing-space) |
| Trait Name | [`trait-name`](#trait-name) |


## <a id="ambiguous-function-call"></a>`ambiguous-function-call`

Enforces that all function calls made from within a namespace are explicit.

When an unqualified function like `strlen()` is called from within a namespace, PHP
performs a runtime fallback check (current namespace -> global namespace). This
ambiguity prevents PHP from performing powerful compile-time optimizations,
such as replacing a call to `strlen()` with the highly efficient `STRLEN` opcode.

Making calls explicit improves readability, prevents bugs, and allows for significant
performance gains in some cases.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

namespace App;

use function strlen;

// OK: Explicitly imported
$length1 = strlen("hello");

// OK: Explicitly global
$length2 = \strlen("hello");

// OK: Explicitly namespaced
$value = namespace\my_function();
```

#### Incorrect code

```php
<?php

namespace App;

// Ambiguous: could be App\strlen or \strlen
$length = strlen("hello");
```


## <a id="array-style"></a>`array-style`

Suggests using the short array style `[..]` instead of the long array style `array(..)`,
or vice versa, depending on the configuration. The short array style is more concise and
is the preferred way to define arrays in PHP.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |
| `style` | `string` | `"short"` |

### Examples

#### Correct code

```php
<?php

// By default, `style` is 'short', so this snippet is valid:
$arr = [1, 2, 3];
```

#### Incorrect code

```php
<?php

// By default, 'short' is enforced, so array(...) triggers a warning:
$arr = array(1, 2, 3);
```


## <a id="assertion-style"></a>`assertion-style`

Enforces a consistent style for PHPUnit assertion calls within test methods.

Maintaining a consistent style (e.g., always using `static::` or `$this->`)
improves code readability and helps enforce team-wide coding standards in test suites.
This rule can be configured to enforce the preferred style.


### Requirements

- **Integration:** `PHPUnit`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `style` | `string` | `"static"` |

### Examples

#### Correct code

```php
<?php
// configured style: "static"
final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        static::assertTrue(true);
    }
}
```

#### Incorrect code

```php
<?php
// configured style: "static"
final class SomeTest extends TestCase
{
    public function testSomething(): void
    {
        $this->assertTrue(true); // Incorrect style
        self::assertFalse(false); // Incorrect style
    }
}
```


## <a id="block-statement"></a>`block-statement`

Enforces that `if`, `else`, `for`, `foreach`, `while`, `do-while` statements always use a block
statement body (`{ ... }`) even if they contain only a single statement.

This improves readability and prevents potential errors when adding new statements.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

if (true) {
    echo "Hello";
}

for ($i = 0; $i < 10; $i++) {
    echo $i;
}
```

#### Incorrect code

```php
<?php

if (true)
    echo "Hello";

for ($i = 0; $i < 10; $i++)
    echo $i;
```


## <a id="braced-string-interpolation"></a>`braced-string-interpolation`

Enforces the use of curly braces around variables within string interpolation.

Using curly braces (`{$variable}`) within interpolated strings ensures clarity and avoids potential ambiguity,
especially when variables are followed by alphanumeric characters. This rule promotes consistent and predictable code.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

$a = "Hello, {$name}!";
$b = "Hello, {$name}!";
$c = "Hello, {$$name}!";
$d = "Hello, {${$object->getMethod()}}!";
```

#### Incorrect code

```php
<?php

$a = "Hello, $name!";
$b = "Hello, ${name}!";
$c = "Hello, ${$name}!";
$d = "Hello, ${$object->getMethod()}!";
```


## <a id="class-name"></a>`class-name`

Detects class declarations that do not follow class naming convention.

Class names should be in class case, also known as PascalCase.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `psr` | `boolean` | `true` |

### Examples

#### Correct code

```php
<?php

class MyClass {}
```

#### Incorrect code

```php
<?php

class my_class {}

class myClass {}

class MY_CLASS {}
```


## <a id="constant-name"></a>`constant-name`

Detects constant declarations that do not follow constant naming convention.

Constant names should be in constant case, also known as UPPER_SNAKE_CASE.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

const MY_CONSTANT = 42;

class MyClass {
    public const int MY_CONSTANT = 42;
}
```

#### Incorrect code

```php
<?php

const myConstant = 42;
const my_constant = 42;
const My_Constant = 42;

class MyClass {
    public const int myConstant = 42;
    public const int my_constant = 42;
    public const int My_Constant = 42;
}
```


## <a id="enum-name"></a>`enum-name`

Detects enum declarations that do not follow class naming convention.

Enum names should be in class case, also known as PascalCase.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

enum MyEnum {}
```

#### Incorrect code

```php
<?php

enum my_enum {}
enum myEnum {}
enum MY_ENUM {}
```


## <a id="function-name"></a>`function-name`

Detects function declarations that do not follow camel or snake naming convention.

Function names should be in camel case or snake case, depending on the configuration.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `camel` | `boolean` | `false` |
| `either` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

function my_function() {}
```

#### Incorrect code

```php
<?php

function MyFunction() {}

function My_Function() {}
```


## <a id="interface-name"></a>`interface-name`

Detects interface declarations that do not follow class naming convention.

Interface names should be in class case and suffixed with `Interface`, depending on the configuration.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `psr` | `boolean` | `true` |

### Examples

#### Correct code

```php
<?php

interface MyInterface {}
```

#### Incorrect code

```php
<?php

interface myInterface {}
interface my_interface {}
interface MY_INTERFACE {}
```


## <a id="lowercase-keyword"></a>`lowercase-keyword`

Enforces that PHP keywords (like `if`, `else`, `return`, `function`, etc.) be written
in lowercase. Using uppercase or mixed case is discouraged for consistency and readability.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

if (true) {
    echo "All keywords in lowercase";
} else {
    return;
}
```

#### Incorrect code

```php
<?PHP

IF (TRUE) {
    ECHO "Keywords not in lowercase";
} ELSE {
    RETURN;
}
```


## <a id="lowercase-type-hint"></a>`lowercase-type-hint`

Enforces that PHP type hints (like `void`, `bool`, `int`, `float`, etc.) be written
in lowercase. Using uppercase or mixed case is discouraged for consistency
and readability.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

function example(int $param): void {
    return;
}
```

#### Incorrect code

```php
<?php

function example(Int $param): VOID {
    return;
}
```


## <a id="no-alias-function"></a>`no-alias-function`

Detects usage of function aliases (e.g., `diskfreespace` instead of `disk_free_space`)
and suggests calling the canonical (original) function name instead.
This is primarily for consistency and clarity.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

// 'disk_free_space' is the proper name instead of 'diskfreespace'
$freeSpace = disk_free_space("/");
```

#### Incorrect code

```php
<?php

// 'diskfreespace' is an alias for 'disk_free_space'
$freeSpace = diskfreespace("/");
```


## <a id="no-hash-comment"></a>`no-hash-comment`

Detects shell-style comments ('#') in PHP code. Double slash comments ('//') are preferred
in PHP, as they are more consistent with the language's syntax and are easier to read.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// This is a good comment.
```

#### Incorrect code

```php
<?php

# This is a shell-style comment.
```


## <a id="no-php-tag-terminator"></a>`no-php-tag-terminator`

Discourages the use of `?><?php` as a statement terminator. Recommends using a semicolon
(`;`) instead for clarity and consistency.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

echo "Hello World";
```

#### Incorrect code

```php
<?php

echo "Hello World" ?><?php
```


## <a id="no-trailing-space"></a>`no-trailing-space`

Detects trailing whitespace at the end of comments. Trailing whitespace can cause unnecessary
diffs and formatting issues, so it is recommended to remove it.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

// This is a good comment.
```

#### Incorrect code

```php
<?php

// This is a comment with trailing whitespace.
```


## <a id="trait-name"></a>`trait-name`

Detects trait declarations that do not follow class naming convention.
Trait names should be in class case and suffixed with `Trait`, depending on the configuration.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `psr` | `boolean` | `true` |

### Examples

#### Correct code

```php
<?php

trait MyTrait {}
```

#### Incorrect code

```php
<?php

trait myTrait {}
trait my_trait {}
trait MY_TRAIT {}
```

