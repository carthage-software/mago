---
title: Format Ignore
---

# Format Ignore

Sometimes you need to preserve specific formatting that the formatter would otherwise change. Mago provides several comment markers to control formatting behavior at different granularities.

## Available Markers

| Marker | Scope | Description |
| :----- | :---- | :---------- |
| `@mago-format-ignore` | File | Skips formatting for the entire file |
| `@mago-format-ignore-next` | Single statement/member | Preserves the next statement or class member |
| `@mago-format-ignore-start` / `@mago-format-ignore-end` | Region | Preserves all code within the region |

All markers also support the `@mago-formatter-` prefix (e.g., `@mago-formatter-ignore-next`).

## File-Level Ignore

To skip formatting for an entire file, add a comment containing `@mago-format-ignore` anywhere in the file:

```php
<?php
// @mago-format-ignore

// This entire file will not be formatted
$a=1;  $b=2;  $c=3;
```

## Ignore Next Statement

The `@mago-format-ignore-next` marker preserves the formatting of the immediately following statement or class member. This is perfect for preserving intentional formatting like aligned arrays or matrices:

```php
<?php

$formatted = 'normal';

// @mago-format-ignore-next
const GRID = [
  [1, 2, 3], [1, 2, ], [0,    0],
  [0,    0], [1,   3], [1, 1, 0]
];

$alsoFormatted = 'normal';
```

### Class Members

The ignore-next marker works with all class member types:

```php
<?php

class Example
{
    // @mago-format-ignore-next
    public const   MATRIX   =   [[1,2], [3,4]];

    // @mago-format-ignore-next
    public    $alignedProperty   =   123;

    // @mago-format-ignore-next
    public function   preservedMethod()   {   return 1;   }
}
```

It also works in traits, interfaces, and enums:

```php
<?php

enum Status: int
{
    // @mago-format-ignore-next
    case   PENDING   =   1;

    case Active = 2;
}

interface MyInterface
{
    // @mago-format-ignore-next
    public function   foo(   $a  ,   $b  )  ;
}

trait MyTrait
{
    // @mago-format-ignore-next
    public    $prop   =   123;
}
```

## Region Ignore

For preserving larger sections of code, use the start/end markers:

```php
<?php

$formatted = 'normal';

// @mago-format-ignore-start
$a=1;
$b=2;
$c=3;
// @mago-format-ignore-end

$alsoFormatted = 'normal';
```

### Class Member Regions

Region markers also work within class bodies:

```php
<?php

class Example
{
    public const FORMATTED = 1;

    // @mago-format-ignore-start
    public const   A   =   1;
    public const   B   =   2;
    public    $prop   =   123;
    public function   foo()   {   return 1;   }
    // @mago-format-ignore-end

    public const ALSO_FORMATTED = 3;
}
```

### Unclosed Regions

If a region is started but not ended, the formatter will preserve everything from the start marker to the end of the file:

```php
<?php

$formatted = 'normal';

// @mago-format-ignore-start
$a=1;
$b=2;
// Everything after the start marker is preserved
```

## Comment Styles

All markers can be used with any PHP comment style:

```php
<?php

// @mago-format-ignore-next
$singleLine = 1;

/* @mago-format-ignore-next */
$blockComment = 2;

/**
 * @mago-format-ignore-next
 */
$docblock = 3;

# @mago-format-ignore-next
$hashComment = 4;
```

## Common Use Cases

### Aligned Data Tables

```php
<?php

// @mago-format-ignore-next
const LOOKUP_TABLE = [
    'short'    => 1,
    'medium'   => 10,
    'long'     => 100,
    'very_long' => 1000,
];
```

### Matrix/Grid Data

```php
<?php

// @mago-format-ignore-next
const TRANSFORMATION_MATRIX = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
```

### ASCII Art Comments

```php
<?php

// @mago-format-ignore-start
/*
 *     _____
 *    /     \
 *   | () () |
 *    \  ^  /
 *     |||||
 */
// @mago-format-ignore-end
```

### Legacy Code Blocks

```php
<?php

// @mago-format-ignore-start
// Legacy code that shouldn't be touched
function old_function($a,$b,$c) {
    return $a+$b+$c;
}
// @mago-format-ignore-end
```

## Linter Integration

Mago includes linter rules to detect when format-ignore markers are placed in locations where they won't have any effect:

- **`ineffective-format-ignore-region`**: Detects when `@mago-format-ignore-start` markers are inside expressions (like array literals or function call arguments) where they cannot affect formatting.

- **`ineffective-format-ignore-next`**: Detects when `@mago-format-ignore-next` markers are inside expressions rather than before statements.

Example of ineffective usage that the linter will warn about:

```php
<?php

// This marker is inside an array - it won't work!
$arr = [ // @mago-format-ignore-next
    1,
    2,
    3
];
```

The correct approach is to ignore the entire statement:

```php
<?php

// @mago-format-ignore-next
$arr = [
    1,     2,     3
];
```
