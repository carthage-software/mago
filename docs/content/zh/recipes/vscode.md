+++
title = "Visual Studio Code 实用方案"
description = "在 VS Code 中把 Mago 用作 PHP 文件的格式化器。"
nav_order = 10
nav_section = "实用方案"
+++
# Visual Studio Code 实用方案

VS Code 目前还没有官方的 Mago 扩展,因此本方案通过 [Custom Local Formatters](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters) 扩展把 Mago 接进来。

## 前置条件

- 已安装 Mago。如果还没有,请参阅 [安装指南](/guide/installation/)。
- `mago` 可执行文件在你的 `PATH` 上。推荐的安装方式会自动处理这一点;可用 `which mago` 验证。

## 配置

### 安装桥接扩展

1. 打开扩展视图(`Ctrl+Shift+X`)。
2. 搜索 `Custom Local Formatters`。
3. 安装作者为 `jkillian` 的扩展。

### 配置 `settings.json`

1. 打开用户的 `settings.json`。在命令面板(`Ctrl+Shift+P`)中执行 "Open User Settings (JSON)" 即可。
2. 加入下列内容(与你已有的配置合并):

```json
{
    "customLocalFormatters.formatters": [
        {
            "command": "mago format --stdin-input",
            "languages": ["php"]
        }
    ],

    "[php]": {
        "editor.defaultFormatter": "jkillian.custom-local-formatters",
        "editor.formatOnSave": true
    }
}
```

保存文件。如果格式化器没有立刻生效,重启 VS Code。

## 使用

启用 `editor.formatOnSave` 后,每次保存 PHP 文件都会被 Mago 格式化。你也可以从命令面板手动运行 "Format Document"。

## 备选方案:Run On Save

如果你更愿意直接调用 Mago,而不走 VS Code 的格式化器 API,[Run On Save](https://marketplace.visualstudio.com/items?itemName=emeraldwalk.RunOnSave) 是个不错的选择。当项目自带 Mago 二进制时尤其合适,因为命令会在你的工作区内运行,并应用你的 `mago.toml`,包括排除规则。

```json
{
    "emeraldwalk.runonsave": {
        "commands": [
            {
                "match": "\\.php$",
                "cmd": "${workspaceFolder}/vendor/bin/mago fmt ${relativeFile}"
            }
        ]
    }
}
```

保存 PHP 文件后,VS Code 会用你的工作区二进制对该文件运行 Mago。
