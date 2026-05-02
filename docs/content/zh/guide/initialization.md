+++
title = "初始化"
description = "基于项目现有的目录布局,以交互方式生成一份起步用的 mago.toml。"
nav_order = 30
nav_section = "指南"
+++
# 初始化

在项目根目录运行 `mago init` 并回答几个问题。命令会写出一份针对当前项目调好的 `mago.toml`。

```sh
mago init
```

如果存在 `composer.json`,Mago 会询问是否读取它,以预填源码路径、PHP 版本以及 linter 应启用的框架集成。如果没有特别的情况,直接接受建议即可。否则命令会回退到手动引导流程。

## 它会问什么

在没有 `composer.json` 或你选择手动配置时,提示包括:

- **源码路径。** Mago 要分析、lint 和格式化的目录。它们会被写入 `paths` 数组。
- **依赖路径。** Mago 应读取以获取上下文但永远不修改的第三方代码,通常是 `vendor`。会被写入 `includes`。
- **排除项。** 完全跳过的目录或 glob 模式(构建产物、生成文件、缓存等)。会被写入 `excludes`。
- **PHP 版本。** 你的代码所针对的版本,用于语法检查和规则适用性判断。
- **Linter 集成。** 要启用的框架特定规则。从 [集成页面](/tools/linter/integrations/) 列出的项中选择。
- **格式化器预设。** 选择一个预设(Default、PSR-12、Laravel、Drupal),或当场自定义各个格式化选项。

提示完成后,命令会把 `mago.toml` 写入当前工作目录。[配置参考](/guide/configuration/) 文档涵盖了该文件支持的每一个选项。

## 参考

```sh
Usage: mago init
```

| 参数 | 说明 |
| :--- | :--- |
| `-h`, `--help` | 打印帮助并退出。 |

适用于每个 Mago 命令的全局选项,请参阅 [CLI 概览](/fundamentals/command-line-interface/)。全局参数必须放在子命令名之前。
