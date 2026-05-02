+++
title = "格式化器"
description = "格式化器的功能,以及它如何产出确定性输出。"
nav_order = 10
nav_section = "工具"
nav_subsection = "Formatter"
+++
# 格式化器

一款面向 PHP 的确定性格式化器。对一个文件运行它,无论该文件当前的风格如何,你都会得到相同的输出。停止争论空白,开始阅读代码。

## 工作原理

Mago 借鉴了 Prettier、`rustfmt` 和 Black 所使用的“解析后再打印”的思路:

1. 把源代码解析为 AST。
2. 丢弃原有的格式(空白、换行、缩进等所有信息)。
3. 按一组固定的规则从头重新打印 AST,默认遵循 [PER-CS](https://www.php-fig.org/per/coding-style/)。

对于给定的 AST,无论输入风格如何,输出都完全一致。代码的运行时行为被精确保留:AST 是可往返的,只有表层表示发生变化。

## 你能得到什么

- **整个项目一致的风格。**格式化器在设计上就是有主见的。
- **默认 PER-CS**,可选 PSR-12、Laravel、Drupal 风格的预设。
- **安全。**格式化器只会做不会改变程序行为的修改。
- **快速。**Rust 内核与基于 arena 的流水线,使得格式化阶段在大多数项目上远低于一秒。

## 接下来读什么

- [使用](/tools/formatter/usage/):如何运行 `mago format`。
- [格式化忽略](/tools/formatter/format-ignore/):用于跳过文件、区域或单条语句格式化的注解。
- [配置参考](/tools/formatter/configuration-reference/):你可以设置的所有选项。
- [命令参考](/tools/formatter/command-reference/):`mago format` 接受的所有标志。
