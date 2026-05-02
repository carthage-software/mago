+++
title = "常见问题"
description = "关于 Mago 项目以及哪些内容属于、哪些不属于本项目的常见问题。"
nav_order = 10
nav_section = "参考"
+++
# 常见问题

## 为什么叫 "Mago"?

项目最初叫 "fennec",取自北非的耳廓狐。由于与另一款工具发生命名冲突,被迫更名。

我们选了 "Mago",以延续 Carthage Software 的根源。Mago of Carthage 是古迦太基的一位作家,被誉为"农业之父"。正如他耕耘土地,这款工具也旨在帮助开发者耕耘自己的代码库。

这个名字还有一层有用的双关含义。在西班牙语和意大利语中,"mago" 意为"魔术师"或"巫师"。logo 同时呈现这两层含义:一只穿着巫师帽和长袍的耳廓狐,衣服上绣着古迦太基的塔尼特(Tanit)符号。

## Mago 怎么读?

`/ˈmɑːɡoʊ/`,"mah-go"。两个音节:"ma" 像 "mama" 中的发音,"go" 像 "go" 的发音。

## Mago 会实现 LSP 吗?

会。Language Server Protocol 实现计划在 `2.0.0` 版本中推出。最初安排在 `1.0.0`,但被推后,以便 LSP 能够以功能完整的形态发布,而不是一个最小化的初版。

更长篇幅的说明请见博客文章 [Why Mago 1.0.0 Won't Ship With an LSP](https://carthage.software/en/blog/article/Why-Mago-1-0-0-Won-t-Ship-With-an-LSP)。

## Mago 会提供编辑器扩展(VS Code 等)吗?

不会。本项目将专注于实现 LSP 标准,不会维护编辑器特定的扩展。支持 LSP 集成的编辑器(Helix、通过 lspconfig 的 Neovim、搭配通用客户端的 VS Code)都能与 Mago 协同工作。我们鼓励社区构建编辑器特定的封装,并乐于在网站上展示口碑良好的实现。

## Mago 会支持分析器插件吗?

会,但不会早于 `1.0.0`。计划是用 Rust 编写插件,编译为 WASM,在运行时由 Mago 加载。这项工作会在 `1.0.0` 发布之后进行。

## Mago 还计划替代哪些其他 PHP 工具?

更长远的愿景是让 Mago 成为一套完整的 PHP 质量保障与开发工具。格式化器、linter 和分析器是 `1.0.0` 的重点。在此之外,计划中的工具包括:

- 一个 PHP 版本管理器。
- 一个 PHP 扩展安装器。
- 一个用于升级 PHP 版本、框架或库的迁移辅助工具。

## Mago 会实现一个 Composer 替代品吗?

不会。Composer 是一款出色的工具,而它的大部分工作都是 I/O 密集型的。Rust 重写不会带来多少速度提升,反而会割裂生态,而且很难支持 Composer 基于 PHP 的插件架构。

## Mago 会实现一个 PHP 运行时吗?

不会。PHP 运行时极为庞大。即便是非常大规模的尝试(Facebook 的 HHVM、VK 的 KPHP)也难以与 Zend 引擎完全对齐。一个更小的项目无法做得更好,而结果只会让社区进一步割裂。Mago 专注于工具链,不涉足运行时。
