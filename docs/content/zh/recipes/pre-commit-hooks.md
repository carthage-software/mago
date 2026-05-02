+++
title = "Pre-commit hooks 实用方案"
description = "在每次 commit 之前对暂存文件运行 lint、analyze 和格式化。"
nav_order = 40
nav_section = "实用方案"
+++
# Pre-commit hooks 实用方案

在每次 git commit 前自动运行 Mago。下面的示例都只对暂存文件操作,因此即便仓库很大,hook 也能保持快速。

## 快速配置

创建 `.git/hooks/pre-commit` 并赋予可执行权限:

```bash
chmod +x .git/hooks/pre-commit
```

## Hook 配置

挑一个匹配你工作流的方案。

### 自动格式化暂存文件

格式化已暂存的 PHP 文件,并把格式化后的版本重新加入暂存。这是对开发者最丝滑的体验,什么都不用记。

```bash
#!/bin/sh

mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

`--staged` 会找出已暂存的文件并只处理那些。对于 `fmt --staged`,格式化后的文件会被自动重新暂存。对于 `lint --staged` 和 `analyze --staged`,在配合 `--fix` 使用时,被修复的文件会被重新暂存。

### 自动修复并自动格式化暂存文件

这个方案在 lint 步骤上加上 `--fix`。`--fail-on-remaining` 在仍有问题无法自动修复需要人工处理时阻止提交。没有这个参数,`--fix` 即便存在未修复的问题也会以零状态退出。

```bash
#!/bin/sh

mago lint --fix --fail-on-remaining --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the remaining issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

如果想要更激进的修复,使用 `--fix --unsafe` 或 `--fix --potentially-unsafe`:

```bash
mago lint --fix --potentially-unsafe --fail-on-remaining --staged
```

### 在格式漂移时阻止提交

只要任何暂存文件没有正确格式化就拒绝提交,要求开发者手动格式化。

```bash
#!/bin/sh

mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --check
if [ $? -ne 0 ]; then
    echo "Some files are not formatted. Please run 'mago fmt' before committing."
    exit 1
fi

exit 0
```

## Husky

如果你使用 [Husky](https://typicode.github.io/husky/),把命令加到 `.husky/pre-commit`:

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

mago lint --staged
mago analyze --staged
mago fmt --staged
```

## CaptainHook

如果你使用 [CaptainHook](https://docs.captainhook.info/),把动作加到 `captainhook.json`:

```json
{
    "pre-commit": {
        "enabled": true,
        "actions": [
            { "action": "mago lint --staged" },
            { "action": "mago analyze --staged" },
            { "action": "mago fmt --staged" }
        ]
    }
}
```

如果只想要 check-only 变体,把最后一个动作换成 `mago fmt --check`。

## `--staged` 与 `--check` 的对比

| 维度 | `--staged` | `--check` |
| :--- | :--- | :--- |
| 行为 | 格式化暂存文件并重新暂存。 | 报告未格式化的文件;有则失败。 |
| 开发者动作 | 无。 | 检查失败时必须手动运行 `mago fmt`。 |
| 适合 | 想要无感格式化的团队。 | 想要对改动有显式控制的团队。 |
| 部分暂存 | 格式化暂存内容,工作树保持不变。 | 与暂存状态无关。 |
