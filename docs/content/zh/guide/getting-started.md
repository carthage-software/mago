+++
title = "快速开始"
description = "Mago 是什么、能做什么,以及接下来该看哪里。"
nav_order = 10
nav_section = "指南"
+++
# 快速开始

Mago 是一款用 Rust 编写的 PHP 工具链。一个二进制涵盖了通常需要三四个独立工具才能完成的工作流。

它包含:

- 一个 [格式化器](/tools/formatter/overview/),产出确定性的输出,默认遵循 PER-CS。
- 一个 [linter](/tools/linter/overview/),内置覆盖九大类别的精选规则集。许多修复可自动应用。
- 一个 [静态分析器](/tools/analyzer/overview/),在运行前捕获类型错误和逻辑 bug,支持 Psalm 和 PHPStan 注解。
- 一个 [架构 guard](/tools/guard/overview/),用于强制执行依赖规则与结构约定。

整套工具以单一二进制运行,无需 PHP 运行时,无需 Composer 依赖,也无需安装 Java。典型的工作流是这样的:

```sh
mago init           # write a starter mago.toml
mago lint           # surface stylistic and correctness issues
mago format         # rewrite files to match the formatter
mago analyze        # type-check and find logic bugs
```

## 接下来去哪里

- [安装](/guide/installation/) 介绍每一种受支持的安装方式。
- [初始化](/guide/initialization/) 介绍交互式的 `mago init` 配置流程。
- [配置](/guide/configuration/) 是 `mago.toml` 中每个选项的参考。
- [Playground](/playground/) 在浏览器中运行完整的 Mago 分析器,无需安装即可试用。
