+++
title = "Zed 实用方案"
description = "在 Zed 编辑器中把 Mago 用作 PHP 文件的格式化器。"
nav_order = 20
nav_section = "实用方案"
+++
# Zed 实用方案

把 Mago 接入 [Zed](https://zed.dev),让 PHP 文件在保存时被格式化。

## 前置条件

- 已安装 Mago。如果还没有,请参阅 [安装指南](/guide/installation/)。
- `mago` 可执行文件在你的 `PATH` 上。推荐的安装方式会自动处理这一点;可用 `which mago` 验证。

## 配置

打开 Zed 的 `settings.json`(macOS 上是 `Cmd + ,`,Linux 和 Windows 上是 `Ctrl + ,`,然后选择 "Open JSON Settings")。在 `languages` 小节加入 PHP 块,与已有内容合并:

```json
{
    "languages": {
        "PHP": {
            "format_on_save": "on",
            "formatter": {
                "external": {
                    "command": "mago",
                    "arguments": ["format", "--stdin-input", "--stdin-filepath", "{buffer_path}"]
                }
            }
        }
    }
}
```

传入 `--stdin-filepath {buffer_path}` 让 Mago 能把 `mago.toml` 中的 `[source].excludes` 和 `[formatter].excludes` 应用到当前缓冲区,并产出更清晰的错误信息。

## 使用

保存 `.php` 文件,Zed 会通过 Mago 进行格式化。也可以从命令面板(`Cmd + Shift + P` 或 `Ctrl + Shift + P`)运行 "Format Buffer" 来手动触发格式化。
