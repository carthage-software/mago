+++
title = "抑制问题"
description = "如何使用 @mago-expect 和 @mago-ignore 这两条指令来在代码中抑制特定问题。"
nav_order = 30
nav_section = "基础"
+++
# 抑制问题

修复底层问题几乎总是正确的答案。但有时并非如此:遗留代码、误报、有意为之的例外。针对这些情况,Mago 提供了两种放在源码里的指令注释:`@mago-expect` 和 `@mago-ignore`。

它们都使用 `category:code` 的形式,共有三个类别可用:

- `lint`(别名 `linter`)用于 linter 问题。
- `analysis`(别名 `analyzer`、`analyser`)用于分析器问题。
- `guard` 用于架构 guard 问题。

可以用逗号分隔的列表一次抑制多个 code,`(N)` 计数简写用于处理同一个 code 在一行内出现 N 次的情况。

## `@mago-expect`

声明在下一行上预期出现某个具体问题。两条指令中较严格的一条,也是我们默认推荐的。

```php
// @mago-expect lint:no-shorthand-ternary
$result = $value ?: 'default';
```

多个 code:

```php
// @mago-expect lint:no-shorthand-ternary,unused-variable
$result = $value ?: 'default';
```

如果每一个预期 code 都匹配到一个真实问题,这些问题就被抑制。如果有任何预期 code 未能匹配,Mago 会报告 `unfulfilled-expect` 警告,让该指令在底层代码被修复后不会静悄悄地残留下来。

## `@mago-ignore`

抑制下一行或下一个块中列出的 code,但在这些 code 不再触发时不会发出警告。Mago 仍会报告一条 `unused-pragma` 提示,以便你清理,只是级别是提示而不是警告。

```php
// @mago-ignore lint:no-shorthand-ternary
$result = $value ?: 'default';
```

多个 code 的写法相同:

```php
// @mago-ignore lint:no-shorthand-ternary,no-assign-in-condition
if ($result = $value ?: 'default') {
    // 对 $result 进行某些操作
}
```

## 块级抑制

当指令位于一个块(函数、类、`if` 等)前一行时,它覆盖整个块。

```php
// @mago-ignore analysis:missing-return-statement
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
    }
    // 此处缺少 return 语句。
}
```

多 code 列表也是同理:

```php
// @mago-ignore analysis:missing-return-statement,unreachable-code
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
        echo 'This code is unreachable';
    }
}
```

## 抑制 N 次出现

当一行(或被作用域指令覆盖的一个块)多次触发同一个 code 时,反复重复这个 code 很繁琐:

```php
// @mago-expect analysis:mixed-operand,mixed-operand,mixed-operand
return $a . $b . $c;
```

改用 `(N)` 简写:

```php
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;
```

`N` 必须是正整数。`code(1)` 等价于直接写 `code`。形如 `(0)`、`(abc)` 或括号不匹配的格式错误后缀会被当作 code 名的一部分,无法匹配。

计数可以与正常的逗号列表混用:

```php
// @mago-expect analysis:mixed-operand(2),unused-variable
```

### 未达成的计数

如果实际匹配到的问题比预期少,Mago 会报告 `unfulfilled-expect`,并且自动修复会下调计数,而不是直接移除指令(那会让原本被匹配上的问题重新出现):

```php
// 之前:预期 3 次匹配,实际只有 2 次。
// @mago-expect analysis:mixed-operand(3)
return $a . $b;

// 修复后:计数下调,保留 2 次真实匹配继续被抑制。
// @mago-expect analysis:mixed-operand(2)
return $a . $b;
```

### 行级与块级语义

对于行级指令,直接使用一个 code 最多抑制一次出现;`(N)` 把上限提升到 `N`。

对于块级(作用域)指令,直接使用一个 code 会抑制块内所有匹配的出现。加上 `(N)` 会把抑制上限限制为 `N` 次,因此第 `N+1` 次匹配的问题仍会被报告。这在你想确保不要冒出超过预期数量的新问题时很有用。

## 抑制所有问题:`all`

特殊的 `all` 一次抑制所有问题。请慎用:它也会隐藏后来新增的代码所引入的任何问题。

在单个类别内:

```php
// @mago-ignore lint:all
$result = $value ?: ($x == true ? 'yes' : 'no');

// @mago-expect analysis:all
function legacy_code(): string {
    if (rand(0, 1)) {
        return 'foo';
    }
}
```

跨所有类别:

```php
// @mago-ignore all
$result = eval($value) ?: 'default';
```

放在一个块上方的 docblock 中,它会覆盖整个块:

```php
/**
 * @mago-ignore all
 */
function legacy_code(): string {
    // 此处所有 linter、analyzer 和 guard 问题均被抑制。
}
```

只要可能,优先使用具体的 code。`all` 是一把钝器,会掩盖你本来想看到的新问题。

## expect 与 ignore 的取舍

- `@mago-expect` 是合理的默认。它保证你在底层问题被修复后能听到对应通知。
- `@mago-ignore` 是更轻量的选项,适合不那么关键的抑制,或你能接受指令默默残留的场景。

## 示例

```php
// 抑制一个 guard 问题
// @mago-expect guard:disallowed-use
use App\Infrastructure\SomeForbiddenClass;

// 抑制一个 lint 问题
// @mago-expect lint:no-shorthand-ternary
$result = $condition ?: 'default';

// 抑制整个函数的若干问题
// @mago-expect analysis:missing-return-statement,impossible-condition
function complexFunction(): string {
    if (false) {
        return 'never reached';
    }
}

// 下一行同一 code 出现三次
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;

// 单行上所有 lint 问题
// @mago-ignore lint:all
$result = $value ?: ($x == true ? 'yes' : 'no');

// 针对一个遗留函数,抑制所有问题
// @mago-ignore all
function legacyFunction(): string {
    // 此处所有问题均被抑制。
}
```
