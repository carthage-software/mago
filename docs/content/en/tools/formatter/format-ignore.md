+++
title = "Format ignore"
description = "Comment markers that opt code out of formatting at the file, region, or single-statement level."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Format ignore

Sometimes you need to keep formatting the formatter would otherwise change: aligned matrices, ASCII art, legacy snippets you do not want touched. Mago has three pragmas for that.

## The pragmas

| Marker | Scope | Effect |
| :--- | :--- | :--- |
| `@mago-format-ignore` | File | Skip formatting for the whole file. |
| `@mago-format-ignore-next` | One statement or class member | Preserve the next statement or member exactly. |
| `@mago-format-ignore-start` / `@mago-format-ignore-end` | Region | Preserve everything between the two markers. |

All three accept the longer prefix `@mago-formatter-` as well, so `@mago-formatter-ignore-next` is identical to `@mago-format-ignore-next`.

## File-level ignore

Place a comment containing `@mago-format-ignore` anywhere in the file and the formatter leaves the entire file alone.

```php
<?php
// @mago-format-ignore

$a=1; $b=2; $c=3;
```

## Ignore the next statement

Useful when you want one carefully-aligned construct to survive untouched. The marker preserves the immediately following statement or class member.

```php
<?php

$formatted = 'normal';

// @mago-format-ignore-next
const GRID = [
    [1, 2, 3], [1, 2, ], [0, 0],
    [0, 0],    [1, 3],   [1, 1, 0]
];

$alsoFormatted = 'normal';
```

The same marker works on every kind of class member:

```php
<?php

class Example
{
    // @mago-format-ignore-next
    public const MATRIX = [[1,2], [3,4]];

    // @mago-format-ignore-next
    public $alignedProperty = 123;

    // @mago-format-ignore-next
    public function preservedMethod() { return 1; }
}
```

It also works in traits, interfaces, and enums:

```php
<?php

enum Status: int
{
    // @mago-format-ignore-next
    case PENDING = 1;

    case Active = 2;
}

interface MyInterface
{
    // @mago-format-ignore-next
    public function foo( $a , $b ) ;
}

trait MyTrait
{
    // @mago-format-ignore-next
    public $prop = 123;
}
```

## Ignore a region

For larger blocks, bracket the region with the start and end markers:

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

Region markers also work inside class bodies:

```php
<?php

class Example
{
    public const FORMATTED = 1;

    // @mago-format-ignore-start
    public const A = 1;
    public const B = 2;
    public $prop = 123;
    public function foo() { return 1; }
    // @mago-format-ignore-end

    public const ALSO_FORMATTED = 3;
}
```

If a region is opened but never closed, the formatter preserves everything from the start marker to the end of the file.

## Comment style

All markers work with every PHP comment style:

```php
<?php

// @mago-format-ignore-next
$lineComment = 1;

/* @mago-format-ignore-next */
$blockComment = 2;

/**
 * @mago-format-ignore-next
 */
$docblock = 3;

# @mago-format-ignore-next
$hashComment = 4;
```

## Common uses

### Aligned data tables

```php
<?php

// @mago-format-ignore-next
const LOOKUP_TABLE = [
    'short'     => 1,
    'medium'    => 10,
    'long'      => 100,
    'very_long' => 1000,
];
```

### Matrices

```php
<?php

// @mago-format-ignore-next
const TRANSFORMATION_MATRIX = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
```

### ASCII art

```php
<?php

// @mago-format-ignore-start
/*
 *   _____
 *  /     \
 * | () () |
 *  \  ^  /
 *   |||||
 */
// @mago-format-ignore-end
```

### Legacy code

```php
<?php

// @mago-format-ignore-start
function old_function($a,$b,$c) {
    return $a+$b+$c;
}
// @mago-format-ignore-end
```

## Linter rules

The linter has two rules that catch ineffective placements:

- `ineffective-format-ignore-region`: catches `@mago-format-ignore-start` markers inside an expression (an array literal, a function-call argument list) where the marker cannot influence formatting.
- `ineffective-format-ignore-next`: catches `@mago-format-ignore-next` markers inside an expression rather than before a statement or member.

The bad case looks like this:

```php
<?php

// The marker is inside an array literal, has no effect.
$arr = [ // @mago-format-ignore-next
    1,
    2,
    3
];
```

The fix is to put the marker before the whole statement:

```php
<?php

// @mago-format-ignore-next
$arr = [
    1, 2, 3,
];
```
