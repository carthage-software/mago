+++
title = "格式化器命令参考"
description = "mago format 接受的所有标志。"
nav_order = 40
nav_section = "工具"
nav_subsection = "Formatter"
+++
# 命令参考

```sh
Usage: mago format [OPTIONS] [PATH]...
```

`mago fmt` 是 `mago format` 的别名,两者均可使用。

全局标志必须放在 `format` 之前。完整的全局标志列表见 [CLI 概述](/fundamentals/command-line-interface/)。

## 参数

| 参数 | 说明 |
| :--- | :--- |
| `[PATH]...` | 要格式化的文件或目录。提供时,本次运行将以这些路径替代 `mago.toml` 中的 `paths`。 |

```sh
mago fmt src/index.php tests/
```

## 选项

| 标志 | 说明 |
| :--- | :--- |
| `--dry-run`, `-d` | 打印将要进行的变更的统一 diff,但不写入任何内容。 |
| `--check`, `-c` | 验证每个源文件是否都已格式化。匹配时退出 `0`,任何文件需要变更时退出 `1`。 |
| `--stdin-input`, `-i` | 从 stdin 读取源代码,格式化后将结果打印到 stdout。 |
| `--stdin-filepath <PATH>` | stdin 缓冲区的逻辑路径。需要 `--stdin-input`。会与 `source.excludes` 和 `formatter.excludes` 进行匹配;若命中,输入会原样回显。同时会替换诊断信息中的 `<stdin>`。 |
| `--staged`, `-s` | 仅格式化 git 中已暂存的文件,并将其重新暂存。为 pre-commit 钩子设计。 |
| `-h`, `--help` | 打印帮助并退出。 |
