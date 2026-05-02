+++
title = "Linter 使用"
description = "驱动 mago lint 的常见方式,包括自动修复和运行单条规则。"
nav_order = 20
nav_section = "工具"
nav_subsection = "Linter"
+++
# 使用

入口是 `mago lint`。它会针对 `mago.toml` 中声明的源文件(或你在命令行传入的参数)运行 linter。

## 对整个项目运行 lint

```sh
mago lint
```

Mago 会并行扫描项目并报告它发现的每一个问题。

## 应用自动修复

大多数规则提供了安全的修复。要就地改写受影响的文件:

```sh
mago lint --fix
```

要以统一 diff 形式预览修复,而不写入磁盘:

```sh
mago lint --fix --dry-run
```

要对修复器改写过的每个文件再运行一次格式化器,追加 `--format-after-fix`:

```sh
mago lint --fix --format-after-fix
```

不那么安全的修复需要明确开启。使用 `--potentially-unsafe` 启用那些可能需要快速复核的修复,使用 `--unsafe` 启用那些可能改变行为的修复。结合 `--dry-run`,你可以在提交前精确看到会发生什么变化。

## 运行单条规则(或几条)

`--only` 只运行列出的规则,跳过其余规则。比运行完整目录更快,在逐步采用 linter 时很有用。

```sh
mago lint --only no-empty
mago lint --only no-empty,use-compound-assignment
```

如果你希望所有规则都运行,但只查看某一部分编码的问题,请改用 `--retain-code`。完整的报告控制标志列表见[报告和修复选项](/fundamentals/shared-reporting-options/)。

## 对指定文件运行 lint

在子命令后传入路径,可将本次运行限定在这些文件或目录上。在针对暂存改动的 pre-commit 钩子中很有用。

```sh
mago lint src/Service/PaymentProcessor.php
mago lint src/Service tests/Unit
```

完整的标志列表见[命令参考](/tools/linter/command-reference/)。
