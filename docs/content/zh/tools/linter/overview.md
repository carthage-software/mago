+++
title = "Linter"
description = "linter 的功能,它与分析器的区别,以及接下来的阅读路径。"
nav_order = 10
nav_section = "工具"
nav_subsection = "Linter"
+++
# Linter

Mago 的 linter 是一个精心整理的规则目录,用于发现代码风格问题、不一致以及代码异味。大多数问题都附带自动修复,因此你可以用一条命令清理大型代码库。

## Linter 与分析器的对比

两个工具都用于发现问题,但它们工作在不同的层面。

**linter** 关注代码的*形态*。它强制执行团队约定,标记冗余结构,并建议更现代的语法。它不需要知道代码在运行时具体做什么,只需要看源码长什么样。

**分析器**会构建整个代码库的语义模型。它知道函数返回什么类型,类有哪些属性,什么可以抛出异常。它能发现逻辑上的不可能,例如在某个类型上调用根本不存在的方法。

如果把代码比作一篇文章,linter 负责语法校对,而分析器负责事实核查。

## 语义检查器

Mago 分三个阶段处理文件:解析、语义检查、lint。

解析器有意做得宽容。它能读取标准 PHP 编译器会拒绝的语法,包括来自未来 PHP 版本的特性。语义检查器是第二阶段,用于捕获那些被宽容解析器放过、但 PHP 会判定为致命错误的问题:

- 无效的枚举支持类型,例如 `enum Foo: array {}`。
- 当前所配置的 PHP 版本不支持的特性,例如在 PHP 8.4 之前使用属性钩子。

使用 `--semantics` 仅运行解析器和语义检查器:

```sh
mago lint -s
```

它是 `php -l` 的一个更快、更彻底的替代品,也是在启用完整规则目录之前,把 Mago 介绍给一个代码库的低门槛方式。

## linter 提供了什么

- **速度。**Rust 内核与基于 arena 的流水线,使得 lint 阶段在大多数项目上远低于一秒。
- **逐规则配置。**每条规则都可以启用、禁用或调整严重等级。部分规则还带有自身的选项。
- **自动修复。**许多规则提供了安全的修复,传入 `--fix`,Mago 就会改写受影响的文件。不太安全的类别需要通过显式标志开启。
- **框架集成。**针对 Symfony、Laravel、PHPUnit、Doctrine、WordPress 以及其他众多项目的可选规则集。按项目启用,详见[集成页面](/tools/linter/integrations/)。

## 接下来读什么

- [使用](/tools/linter/usage/):如何运行 `mago lint`。
- [集成](/tools/linter/integrations/):启用框架专属检查。
- [规则](/tools/linter/rules/):每条规则的完整参考。
- [配置参考](/tools/linter/configuration-reference/):Mago 在 `[linter]` 下接受的所有选项。
- [命令参考](/tools/linter/command-reference/):`mago lint` 接受的所有标志。
