+++
title = "Baseline"
description = "为已存在的问题保存快照,让 Mago 只标记新出现的问题。提供两种变体,在精度与韧性之间提供不同的取舍。"
nav_order = 20
nav_section = "基础"
+++
# Baseline

baseline 文件记录了你代码库当前存在的所有问题,并告诉 Mago 在后续运行中忽略它们。在 baseline 之后引入的新问题仍会被标记。这在你为已经积累了几百上千个问题的项目接入 Mago,或在多个 PR 中分阶段收紧规则时非常有用。

## 每个工具一份文件

linter 和分析器各自维护自己的 baseline,因为它们报告的问题不同。常见命名:

- Linter: `lint-baseline.toml`
- 分析器: `analysis-baseline.toml`

`mago ast` 命令报告的是解析错误,不支持 baseline。

## 生成 baseline

```sh
mago lint --generate-baseline --baseline lint-baseline.toml
mago analyze --generate-baseline --baseline analysis-baseline.toml
```

命令会运行对应工具,收集找到的所有问题,并把它们序列化到指定的 TOML 文件中。

## 使用 baseline

```sh
mago lint --baseline lint-baseline.toml
mago analyze --baseline analysis-baseline.toml
```

启用 baseline 时,Mago 会:

1. 找出当前代码中的所有问题。
2. 与 baseline 进行比对。
3. 抑制匹配的问题。
4. 只报告剩下的问题。

也可以在 `mago.toml` 中设置 baseline 路径,这样就不必每次都传 `--baseline`:

```toml
[linter]
baseline = "lint-baseline.toml"

[analyzer]
baseline = "analysis-baseline.toml"
```

## 两种变体

Mago 支持两种 baseline 形态,它们在精度与韧性之间有不同取舍。

### Loose(默认)

按 `(file, code, message)` 对问题进行分组,并存储一个计数。对行号偏移有较强韧性:即便你重新格式化或在某个问题上方插入代码,只要同类问题仍然出现,baseline 仍能匹配。

```toml
variant = "loose"

[[issues]]
file    = "src/Service/PaymentProcessor.php"
code    = "possibly-null-argument"
message = "Argument #1 of `process` expects `Order`, but `?Order` was given."
count   = 2
```

### Strict

按问题存储精确的行号区间。精度高,但每次行号变化时 baseline 都会过期,因此你需要频繁重新生成。

```toml
variant = "strict"

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 42
end_line   = 42

[[entries."src/Service/PaymentProcessor.php".issues]]
code = "possibly-null-argument"
start_line = 87
end_line   = 90
```

### 如何选择

| 变体 | 适合 | 取舍 |
| :--- | :--- | :--- |
| Loose | 大多数项目、CI 流水线 | 对重构有韧性,精度较低。 |
| Strict | 需要精确行号追踪时 | 精确,但需要频繁重新生成。 |

在 `mago.toml` 中为新生成的 baseline 文件指定变体:

```toml
[linter]
baseline = "lint-baseline.toml"
baseline-variant = "loose"   # 或 "strict"

[analyzer]
baseline = "analysis-baseline.toml"
baseline-variant = "loose"
```

该设置只影响生成。读取已有 baseline 时,Mago 会从文件的 `variant` 头信息中检测变体。

### 向后兼容

由更早的 Mago 版本(在引入 variant 支持之前)生成的 baseline 文件没有 `variant` 头部信息。Mago 会把这类文件视为 strict,并打印一条警告,建议你重新生成以获得该头部信息。

## 临时跳过 baseline

```sh
mago lint --ignore-baseline
mago analyze --ignore-baseline
```

当你想看到当前被 baseline 抑制的所有问题(例如打算清理其中一部分时)很有用。

## 保持 baseline 整洁

当你修复了 baseline 中的某个问题后,对应条目就成了死条目。Mago 会检测到并就过期条目发出警告。重新生成以清理:

```sh
mago lint --generate-baseline --baseline lint-baseline.toml
```

传 `--backup-baseline` 可以在覆盖前把旧文件保留为 `lint-baseline.toml.bkp`。

## JSON Schema

如果你正在构建需要解析或生成 baseline 文件的工具或 IDE 集成,可以获取 schema:

```sh
mago config --schema --show baseline
```

输出是一份覆盖两种变体的 JSON Schema (draft 2020-12)。
