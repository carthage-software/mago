+++
title = "Guard 命令参考"
description = "mago guard 接受的所有标志。"
nav_order = 30
nav_section = "工具"
nav_subsection = "Guard"
+++
# 命令参考

```sh
Usage: mago guard [OPTIONS] [PATHS]...
```

全局标志必须放在 `guard` 之前。完整的全局标志列表见 [CLI 概述](/fundamentals/command-line-interface/)。

## 参数

| 参数 | 说明 |
| :--- | :--- |
| `[PATHS]...` | 要检查的文件或目录。提供时,本次运行将以这些路径替代 `mago.toml` 中的 `paths`。 |

## 模式选择

这些标志选择 guard 的哪一部分运行,它们互斥。

| 标志 | 说明 |
| :--- | :--- |
| `--structural` | 仅运行结构检查(命名、修饰符、继承)。 |
| `--perimeter` | 仅运行边界检查(依赖边界、分层限制)。 |

如果两个标志都未设置,会同时运行两部分,等同于配置中的 `mode = "default"`。这些标志会覆盖配置中的 `mode`。如果标志与配置中的 mode 一致,guard 会打印一条冗余警告。

## 其他选项

| 标志 | 说明 |
| :--- | :--- |
| `--no-stubs` | 跳过内置的 PHP 与库 stub。仅在确实需要时使用。 |
| `--stdin-input` | 从 stdin 读取文件内容,使用单一路径参数进行 baseline 查找和报告。用于编辑器集成。 |
| `--substitute <ORIG=TEMP>` | 在本次调用中以另一个文件替换某个宿主文件。用于变异测试。可重复使用。 |
| `-h`, `--help` | 打印帮助并退出。 |

报告、修复和 baseline 的共享标志记录在[报告和修复选项](/fundamentals/shared-reporting-options/)页面。自动修复目前对 guard 问题没有实际意义,但为了与其他工具保持一致,仍接受这些标志。

## 从 stdin 读取

适用于通过管道传入未保存缓冲区内容的编辑器集成:

```sh
cat src/Example.php | mago guard --stdin-input src/Example.php
```

需要恰好一个路径参数。它会作为相对于工作区的文件名,用于 baseline 匹配和诊断。路径会被规范化,因此 `./src/Example.php` 与 `src/Example.php` 等价。与 `--substitute` 互斥。

## 替换文件

`--substitute ORIG=TEMP` 在单次运行期间用另一个文件替换某个宿主文件,且不会向磁盘写入任何内容。它为变异测试框架而设计,这些框架会生成源文件的变异副本,并希望 guard 针对项目其余部分对该变异进行评估。

```sh
mago guard --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

规则:

- `ORIG` 与 `TEMP` 都必须是绝对路径,且两个文件都必须存在。
- `ORIG` 必须是配置中某个 `paths` 下的宿主文件。第三方依赖或被排除的文件不能被替换。
- 该标志可以重复使用,以便一次替换多个文件。
- 与 `--stdin-input` 互斥。

在内部,`TEMP` 会被加入宿主路径,`ORIG` 会被加入本次运行的排除项,因此依赖分析仍然可以看到该变异。报告的问题引用的是 `TEMP` 而非 `ORIG`。
