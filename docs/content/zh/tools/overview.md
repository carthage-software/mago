+++
title = "工具"
description = "Mago 各项工具的功能介绍及深入了解的入口。"
nav_order = 10
nav_section = "工具"
+++
# 工具

Mago 是一个二进制文件,集成了四种工具。它们共享配置、解析器和运行时,因此你可以任意组合使用,不会为没用到的工具付出额外代价。

## [格式化器](/tools/formatter/overview/)

一个确定性的代码格式化器。默认输出稳定、规范的代码,遵循 [PER-CS](https://www.php-fig.org/per/coding-style/),并支持 PSR-12、Laravel 和 Drupal 风格的预设。没有配置上的左右摇摆,也没有无谓的争论。

## [Linter](/tools/linter/overview/)

一个精心整理的规则目录,涵盖正确性、一致性、清晰度、冗余、安全性以及其他若干方面。大多数问题都附带自动修复。框架集成在此之上添加了针对 Symfony、Laravel、PHPUnit、Doctrine 等的专属规则。

## [分析器](/tools/analyzer/overview/)

一个静态分析引擎,在运行时之前捕获类型错误和逻辑 bug。兼容 Psalm 和 PHPStan 注解,支持泛型、条件类型以及流敏感的类型收窄。

## [架构 guard](/tools/guard/overview/)

强制执行依赖规则和结构约定。当你想禁止某些 `use` 路径、固化分层边界,或断言项目中某一部分的代码绝不会引用另一部分时,这个工具非常有用。
