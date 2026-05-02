+++
title = "升级"
description = "通过 self-update 命令保持 Mago 版本最新,包括锁定到指定版本或与 mago.toml 中的 version 字段同步。"
nav_order = 80
nav_section = "指南"
+++
# 升级

`mago self-update` 会用更新的发行版替换当前正在运行的二进制。适用于通过 shell 脚本、Homebrew、Cargo 或手动下载安装的场景。

> Composer 安装方式不同。Composer 包装会锁定一份与 Composer 包版本匹配的二进制,所以你应使用 `composer update` 而非 `self-update` 来升级 Mago。

## 常见流程

只检查更新而不安装:

```sh
mago self-update --check
```

命令会打印新版本(如果有的话),并在有可用更新时以非零状态退出,这让它可以在 CI 中脚本化使用。

更新到最新版:

```sh
mago self-update                  # interactive confirmation
mago self-update --no-confirm     # skip the prompt
```

锁定到指定版本:

```sh
mago self-update --tag 1.25.2
```

## 与项目的版本锁定同步

如果你的 `mago.toml` 使用了 [版本锁定](/guide/configuration/#version-pinning),你可以把已安装二进制同步到项目期望的版本,而不必自己输入版本号:

```sh
mago self-update --to-project-version
```

对于精确锁定(`version = "1.25.2"`),会直接解析到对应的发布 tag。对于主版本或次版本锁定,Mago 会扫描近期的 GitHub 发布,安装仍然满足锁定的最高版本。所以即便 2.0 已发布,`version = "1"` 仍会安装最新的 1.x。`version = "1.14"` 在 1.19.x 已经流行的情况下,会回退到最新的 1.14.x。

只有在没有任何已发布版本满足锁定时,命令才会失败。

## 参考

```sh
Usage: mago self-update [OPTIONS]
```

| 参数 | 说明 |
| :--- | :--- |
| `--check`, `-c` | 仅检查更新,不进行安装。有可用更新时以非零状态退出。 |
| `--no-confirm` | 跳过交互式确认提示。 |
| `--tag <VERSION>` | 安装指定的发布 tag,而不是最新版本。与 `--to-project-version` 互斥。 |
| `--to-project-version` | 安装项目 `version` 锁定所要求的版本。未设置锁定时失败。与 `--tag` 互斥。 |
| `-h`, `--help` | 打印帮助并退出。 |

全局参数必须放在 `self-update` 之前。完整列表见 [CLI 概览](/fundamentals/command-line-interface/)。
