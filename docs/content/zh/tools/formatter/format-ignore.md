+++
title = "格式化忽略"
description = "可在文件、区域或单条语句级别让代码不被格式化的注释标记。"
nav_order = 30
nav_section = "工具"
nav_subsection = "Formatter"
+++
# 格式化忽略

有时你需要保留格式化器本会更改的格式:对齐的矩阵、ASCII 艺术、不希望被改动的遗留片段。Mago 为此提供了三种注解。

## 这些注解

| 标记 | 作用范围 | 效果 |
| :--- | :--- | :--- |
| `@mago-format-ignore` | 文件 | 跳过整个文件的格式化。 |
| `@mago-format-ignore-next` | 一条语句或一个类成员 | 完整保留下一条语句或下一个成员。 |
| `@mago-format-ignore-start` / `@mago-format-ignore-end` | 区域 | 保留两个标记之间的所有内容。 |

这三个标记同样接受更长的前缀 `@mago-formatter-`,因此 `@mago-formatter-ignore-next` 与 `@mago-format-ignore-next` 等价。

## 文件级忽略

在文件中任意位置放一条包含 `@mago-format-ignore` 的注释,格式化器就会跳过整个文件。

```php
<?php
// @mago-format-ignore

$a=1; $b=2; $c=3;
```

## 忽略下一条语句

适用于希望某一条精心对齐的构造保持不变的场景。该标记会保留紧随其后的语句或类成员。

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

同一标记适用于各种类成员:

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

它在 trait、接口和枚举中同样适用:

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

## 忽略一个区域

对于较大的代码块,使用起止标记将该区域包起来:

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

区域标记同样适用于类体内部:

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

如果一个区域已开启但从未关闭,格式化器会保留从起始标记到文件末尾的所有内容。

## 注释风格

所有标记适用于每一种 PHP 注释风格:

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

## 常见用法

### 对齐的数据表

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

### 矩阵

```php
<?php

// @mago-format-ignore-next
const TRANSFORMATION_MATRIX = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
```

### ASCII 艺术

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

### 遗留代码

```php
<?php

// @mago-format-ignore-start
function old_function($a,$b,$c) {
    return $a+$b+$c;
}
// @mago-format-ignore-end
```

## linter 规则

linter 提供了两条规则,用于捕获无效的标记位置:

- `ineffective-format-ignore-region`:发现位于表达式内部(数组字面量、函数调用实参列表)的 `@mago-format-ignore-start` 标记,在那里它无法影响格式化。
- `ineffective-format-ignore-next`:发现位于表达式内部、而不是位于某条语句或成员之前的 `@mago-format-ignore-next` 标记。

错误用法看起来像这样:

```php
<?php

// The marker is inside an array literal, has no effect.
$arr = [ // @mago-format-ignore-next
    1,
    2,
    3
];
```

正确做法是把标记放到整条语句之前:

```php
<?php

// @mago-format-ignore-next
$arr = [
    1, 2, 3,
];
```
