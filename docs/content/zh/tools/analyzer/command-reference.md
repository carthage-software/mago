+++
title = "分析器命令参考"
description = "mago analyze 接受的所有标志。"
nav_order = 20
nav_section = "工具"
nav_subsection = "Analyzer"
+++
# 命令参考

```sh
Usage: mago analyze [OPTIONS] [PATHS]...
```

`mago analyse` 是 `mago analyze` 的别名,两者均可使用。

全局标志必须放在 `analyze` 之前。完整的全局标志列表见 [CLI 概述](/fundamentals/command-line-interface/)。

## 参数

| 参数 | 说明 |
| :--- | :--- |
| `[PATHS]...` | 要分析的文件或目录。提供时,本次运行将以这些路径替代 `mago.toml` 中的 `paths`。 |

## 分析器专属选项

| 标志 | 说明 |
| :--- | :--- |
| `--no-stubs` | 跳过内置的 PHP 标准库 stub。仅在确实需要时使用。 |
| `--staged` | 仅分析 git 中暂存的文件。在非 git 仓库中运行会失败。 |
| `--stdin-input` | 从 stdin 读取文件内容,使用单一路径参数进行 baseline 查找和报告。用于编辑器集成。 |
| `--substitute <ORIG=TEMP>` | 在本次调用中以另一个文件替换某个宿主文件。用于变异测试。可重复使用。 |
| `--watch` | 持续运行,在文件变化时重新分析。参见[监视模式](#watch-mode)。 |
| `--list-codes` | 以 JSON 列出每一项分析器问题的代码。 |
| `-h`, `--help` | 打印帮助并退出。 |

报告、修复和 baseline 的共享标志记录在[报告和修复选项](/fundamentals/shared-reporting-options/)页面。

## 从 stdin 读取

适用于通过管道传入未保存缓冲区内容的编辑器和 IDE 集成:

```sh
cat src/Example.php | mago analyze --stdin-input src/Example.php
```

需要恰好一个路径参数。它会作为逻辑(相对于工作区的)文件名,用于 baseline 匹配和诊断。路径会被规范化,因此 `./src/Example.php` 与 `src/Example.php` 等价。与 `--staged` 和 `--watch` 互斥。

## 替换文件

`--substitute ORIG=TEMP` 在单次运行期间用另一个文件替换某个宿主文件,且不会向磁盘写入任何内容。它为变异测试框架(Infection 等)而设计,这些框架会生成源文件的变异副本,并希望分析器针对项目其余部分对该变异进行评估。如果分析器在变异文件上报告了新错误,该变异就可以在不运行测试套件的情况下被判定为已被杀死。

```sh
mago analyze --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

规则:

- `ORIG` 与 `TEMP` 都必须是绝对路径,且两个文件都必须存在。
- `ORIG` 必须是配置中某个 `paths` 下的宿主文件。第三方依赖或被排除的文件不能被替换。
- 该标志可以重复使用,以便一次替换多个文件。
- 与 `--stdin-input` 和 `--staged` 互斥。

在内部,`TEMP` 会被加入宿主路径,`ORIG` 会被加入本次运行的排除项,因此跨文件的类型推断仍然可以看到该变异。报告的问题和 baseline 条目引用的是 `TEMP` 而非 `ORIG`。

## 监视模式

`--watch` 让分析器持续运行,在工作区中创建、修改或删除任何 PHP 文件时重新执行分析。

```sh
mago analyze --watch
```

### 自动重启

分析器还会监视那些会改变其自身配置的文件:

- 已加载的 `mago.toml`(或 Mago 选用的任何配置)。
- `[analyzer].baseline` 引用的 baseline 文件。
- `composer.json` 与 `composer.lock`。

任何这些文件发生变化时,分析器会以重新加载的配置重启。因此你可以编辑 `mago.toml` 后保存,下一次分析就会使用新设置,无需手动重启。

如果监视模式启动时不存在任何配置文件,分析器会等待任意一种受支持的配置文件(`mago.toml`、`mago.yaml`、`mago.json` 等)被创建,并在出现时重启。

按 **Ctrl+C** 停止监视。
