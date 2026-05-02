+++
title = "Linter 命令参考"
description = "mago lint 接受的所有标志。"
nav_order = 50
nav_section = "工具"
nav_subsection = "Linter"
+++
# 命令参考

```sh
Usage: mago lint [OPTIONS] [PATH]...
```

全局标志必须放在 `lint` 之前。完整的全局标志列表见 [CLI 概述](/fundamentals/command-line-interface/)。

## 参数

| 参数 | 说明 |
| :--- | :--- |
| `[PATH]...` | 要 lint 的文件或目录。提供时,本次运行将以这些路径替代 `mago.toml` 中的 `paths`。 |

## linter 专属选项

| 标志 | 说明 |
| :--- | :--- |
| `--list-rules` | 列出每条已启用的规则及其说明。 |
| `--json` | 与 `--list-rules` 配合使用,输出机器可读的 JSON。 |
| `--explain <CODE>` | 打印某一条规则的详细文档,例如 `--explain no-redundant-nullsafe`。 |
| `--only <CODE>`, `-o` | 仅运行列出的规则。逗号分隔。覆盖配置。 |
| `--pedantic` | 启用每一条规则,忽略 PHP 版本限制并启用默认禁用的规则。 |
| `--semantics`, `-s` | 仅运行解析 + 语义检查。跳过 lint 规则。 |
| `--staged` | 仅 lint git 中暂存的文件。在非 git 仓库中运行会失败。 |
| `--stdin-input` | 从 stdin 读取文件内容,使用单一路径参数进行 baseline 查找和报告。用于编辑器集成。 |
| `--substitute <ORIG=TEMP>` | 在本次调用中以另一个文件替换某个宿主文件。用于变异测试。可重复使用。 |
| `-h`, `--help` | 打印帮助并退出。 |

报告、修复和 baseline 的共享标志记录在[报告和修复选项](/fundamentals/shared-reporting-options/)页面。

## 从 stdin 读取

当编辑器或 IDE 通过管道传入未保存的缓冲区内容时,你可以对该内容进行 lint,同时仍使用真实文件路径来做 baseline 查找和定位问题:

```sh
cat src/Example.php | mago lint --stdin-input src/Example.php
```

需要恰好一个路径参数。它会作为逻辑(相对于工作区的)文件名,用于 baseline 匹配和诊断。路径会被规范化,因此 `./src/Example.php` 与 `src/Example.php` 等价。与 `--staged` 互斥。

## 替换文件

`--substitute ORIG=TEMP` 在单次运行期间用另一个文件替换某个宿主文件,且不会向磁盘写入任何内容。它为变异测试框架(Infection 等)而设计,这些框架会生成源文件的变异副本,并希望 linter 针对项目其余部分对该变异进行评估。如果 linter 在变异文件上报告了新问题,该变异就可以在不运行测试套件的情况下被判定为已被杀死。

```sh
mago lint --substitute /abs/path/to/src/Foo.php=/tmp/mutation-42.php
```

规则:

- `ORIG` 与 `TEMP` 都必须是绝对路径,且两个文件都必须存在。
- `ORIG` 必须是配置中某个 `paths` 下的宿主文件。第三方依赖或被排除的文件不能被替换。
- 该标志可以重复使用,以便一次替换多个文件。
- 与 `--stdin-input` 和 `--staged` 互斥。

在内部,`TEMP` 会被加入宿主路径,`ORIG` 会被加入本次运行的排除项,因此跨文件的规则仍然可以看到该变异。报告的问题和 baseline 条目引用的是 `TEMP` 而非 `ORIG`。变异测试工具通常会比较干净运行与替换运行之间的问题数量,因此这并不会改变工作流。
