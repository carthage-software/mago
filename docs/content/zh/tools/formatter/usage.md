+++
title = "格式化器使用"
description = "驱动 mago format 的常见方式:就地写入、dry-run、检查模式、stdin、pre-commit。"
nav_order = 20
nav_section = "工具"
nav_subsection = "Formatter"
+++
# 使用

`mago format`(别名 `mago fmt`)是入口。默认情况下,它会就地格式化 `mago.toml` 中声明的所有源文件。

## 格式化整个项目

```sh
mago format
```

文件会被就地改写。可在拉取代码后、提交前或 CI 的某个步骤中运行。

## CI:检查而不改写

在持续集成步骤中,你通常希望验证项目是否已格式化,而不修改任何东西。`--check` 标志正是用于此:

```sh
mago format --check
```

每个文件都已格式化时退出 `0`,任何文件需要变更时退出 `1`。成功时没有输出,因此在正常路径上保持安静。

## 预览变更

要查看格式化器会做什么,而不向磁盘写入任何东西,使用 dry-run:

```sh
mago format --dry-run
```

输出是建议变更的统一 diff。

## 指定文件或目录

在子命令后传入路径,可将本次运行限定在这些范围内:

```sh
mago format src/Service.php
mago format src/ tests/
```

## 从 stdin 读取

适合从编辑器或其他工具通过管道传入缓冲区内容。从 stdin 读取,把格式化结果打印到 stdout。

```sh
cat src/Service.php | mago format --stdin-input
```

编辑器集成还应当传入缓冲区的路径,这样排除规则才能生效,并且解析错误信息中可以显示真实文件名:

```sh
cat src/Service.php | mago format --stdin-input --stdin-filepath src/Service.php
```

如果该路径匹配某个排除模式,输入会原样回显。相对路径和绝对路径都接受。

## Pre-commit(仅暂存文件)

`--staged` 仅格式化 git 中已暂存的文件,然后将其重新暂存。它为 pre-commit 钩子设计,可以避免触碰工作区中未暂存的改动。

```sh
mago format --staged
```

[pre-commit 配方](/recipes/pre-commit-hooks/)给出了完整的钩子配置说明。

完整的标志列表见[命令参考](/tools/formatter/command-reference/)。
