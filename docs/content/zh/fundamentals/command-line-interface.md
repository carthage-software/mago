+++
title = "命令行接口"
description = "全局选项、子命令、环境变量和退出码。"
nav_order = 10
nav_section = "基础"
+++
# 命令行接口

每次调用 Mago 都遵循 `mago [GLOBAL OPTIONS] <SUBCOMMAND>` 的形式。全局选项必须放在子命令之前。

```sh
mago --colors=never lint        # 正确
mago lint --colors=never        # 错误, --colors 是全局选项
```

## 全局选项

下列选项对每个子命令都生效,用于控制运行时、配置发现和输出。

| 参数 | 说明 |
| :--- | :--- |
| `--workspace <PATH>` | 工作空间根目录。默认为当前目录。 |
| `--config <PATH>` | 配置文件路径。未指定时,Mago 会在工作空间、`$XDG_CONFIG_HOME`、`~/.config` 和 `~` 中查找。参见 [发现](/guide/configuration/#discovery)。 |
| `--php-version <VERSION>` | 覆盖配置中的 PHP 版本,例如 `8.2`。 |
| `--threads <NUMBER>` | 覆盖线程数。默认是逻辑 CPU 数。 |
| `--allow-unsupported-php-version` | 允许在 Mago 官方不支持的 PHP 版本上运行。请谨慎使用。 |
| `--no-version-check` | 关闭因已安装二进制与项目锁定版本之间次版本或补丁版本漂移所发出的警告。主版本漂移仍是致命错误。参见 [版本锁定](/guide/configuration/#version-pinning)。 |
| `--colors <WHEN>` | 何时为输出着色:`always`、`never` 或 `auto`(默认)。 |
| `-h`, `--help` | 打印帮助并退出。 |
| `-V`, `--version` | 打印已安装版本并退出。 |

## 环境变量

大多数配置覆盖使用 `MAGO_*` 前缀,详见 [环境变量页面](/guide/environment-variables/)。日常最常设置的两个是:

| 变量 | 用途 |
| :--- | :--- |
| `MAGO_LOG` | tracing 输出的日志过滤器。可选值:`trace`、`debug`、`info`、`warn`、`error`。 |
| `MAGO_EDITOR_URL` | 终端输出中可点击文件路径的 URL 模板。参见 [编辑器集成](/guide/configuration/#editor-integration)。 |

## 子命令

核心工具:

| 命令 | 说明 |
| :--- | :--- |
| [`mago analyze`](/tools/analyzer/command-reference/) | 静态分析:类型错误、逻辑 bug。 |
| [`mago ast`](/guide/inspecting-the-ast/) | 打印 PHP 文件的 AST。 |
| [`mago format`](/tools/formatter/command-reference/) | 格式化 PHP 文件。 |
| [`mago guard`](/tools/guard/command-reference/) | 强制执行架构规则与边界。 |
| [`mago lint`](/tools/linter/command-reference/) | 针对风格、正确性和最佳实践进行 lint。 |

实用命令:

| 命令 | 说明 |
| :--- | :--- |
| [`mago config`](/guide/configuration/) | 打印合并后的配置或其 JSON Schema。 |
| [`mago init`](/guide/initialization/) | 生成一份起步用的 `mago.toml`。 |
| [`mago list-files`](/guide/list-files/) | 列出 Mago 将处理的文件。 |
| [`mago generate-completions`](/guide/generate-completions/) | 打印 shell 补全脚本。 |
| [`mago self-update`](/guide/upgrading/) | 用更新的发行版替换已安装的二进制。 |

## 退出码

| 码 | 含义 |
| :--- | :--- |
| `0` | 成功。未发现问题。 |
| `1` | 发现需要关注的问题。 |
| `2` | 工具错误:配置、I/O、解析失败等。 |
