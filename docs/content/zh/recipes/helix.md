+++
title = "Helix 实用方案"
description = "把 Mago 接入 Helix 编辑器,作为 PHP 的格式化器。"
nav_order = 30
nav_section = "实用方案"
+++
# Helix 实用方案

把 Mago 用作 [Helix 编辑器](https://helix-editor.com/) 中 PHP 文件的格式化器。

## 前置条件

- 已安装 Mago。如果还没有,请参阅 [安装指南](/guide/installation/)。
- `mago` 可执行文件在你的 `PATH` 上。推荐的安装方式会自动处理这一点;你可以用 `which mago` 验证。

## 配置

在 Helix 的 `languages.toml` 中加几行:

- 在 Linux 和 macOS 上,该文件通常位于 `~/.config/helix/languages.toml`。
- 在 Windows 上,通常位于 `%AppData%\helix\languages.toml`。

如果文件不存在就创建它,然后追加:

```toml
[[language]]
name = "php"

formatter = { command = "mago", args = ["format", "--stdin-input"] }
auto-format = true
```

这会覆盖 Helix 默认的 PHP 格式化器,并在保存时自动格式化。

## 使用

启用 `auto-format = true` 后,每次保存(`:write` 或 `:w`)Mago 都会运行。你也可以在命令模式下手动用 `:format`(或 `:fmt`)触发格式化。

要验证配置,打开一个 `.php` 文件,把代码搞乱,然后保存。代码应该立刻归位。
