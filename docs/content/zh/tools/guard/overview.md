+++
title = "Guard"
description = "guard 的功能,以及它的两个部分如何分工。"
nav_order = 10
nav_section = "工具"
nav_subsection = "Guard"
+++
# Guard

`mago guard` 在 PHP 项目中强制执行架构边界和结构约定。它涵盖了与 deptrac 和 arkitect 相同的范围,集成于一个二进制文件中,运行在 Mago 的解析器之上。

该工具有两部分:边界 guard 验证分层之间的依赖,而结构 guard 在符号自身上执行约定。

## 边界 guard

边界 guard 验证依赖边。它确保应用各部分之间只通过你明确允许的方式互相调用,从而让领域层不被基础设施泄露污染,UI 也无法越过应用层。

典型规则:

- `Domain` 层不能依赖任何其他层。
- `UI` 层可以依赖 `Application`,但反过来不行。
- 某个特定模块只允许使用一份获批的库列表。

## 结构 guard

结构 guard 在符号自身上执行约定:它们的名称、修饰符、父类型、attribute,以及它们所在命名空间的形态。

典型规则:

- `App\Http\Controllers` 中的所有类都必须是 `final` 且以 `Controller` 结尾。
- `Domain` 下的接口必须以 `Interface` 结尾。
- 某个命名空间只能包含 `enum` 定义。

## 接下来读什么

- [使用](/tools/guard/usage/):常见命令以及它们的输出形态。
- [命令参考](/tools/guard/command-reference/):`mago guard` 接受的所有标志。
- [配置参考](/tools/guard/configuration-reference/):Mago 在 `[guard]` 下接受的所有选项。
