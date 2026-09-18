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
| [`mago cst`](/guide/inspecting-the-ast/) | 打印 PHP 文件的 CST。 |
| [`mago fix`](#mago-fix) | 运行四个工具并应用修复，直到不再产生更改。 |
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
| `mago version` | 打印 Mago 的版本,与 `--version` 相同。 |

## mago fix

`mago fix [PATHS...]` 按 **guard → analyzer → linter → formatter** 的顺序运行，然后重复此顺序，直到完整一轮不再修改任何文件。每个工具都会读取前一个工具的更改。不指定路径时，命令使用配置中的源码路径。

```sh
mago fix
mago fix src/ tests/ --potentially-unsafe
mago fix --no-analyze --no-guard
```

默认只应用安全修复。命令遵循每个工具的配置、排除规则、代码中的忽略标记和基线。没有可用修复或超出所选安全级别的问题，不会阻止其他工具运行，也不会导致无限循环。

| 参数 | 说明 |
| :--- | :--- |
| `--potentially-unsafe` | 允许安全和可能不安全的修复。 |
| `--unsafe` | 允许所有修复。请仔细检查更改。 |
| `--no-guard` | 跳过 guard。 |
| `--no-analyze` | 跳过 analyzer。 |
| `--no-lint` | 跳过 linter。 |
| `--no-fmt` | 跳过 formatter。 |
| `--ignore-baseline` | 同时修复各工具基线中已忽略的问题。 |
| `--fail-on-remaining` | 修复稳定后仍有问题时，以退出码 `1` 结束。 |
| `--max-passes <NUMBER>` | 完整执行轮数上限，默认 `10`，取值范围为 `1` 到 `256`。 |

默认情况下，只要所选安全级别下没有更多可用修复，命令就会成功，即使仍有问题。禁用全部四个工具时，命令不做任何更改并成功退出。

如果修复使文件反复回到先前的状态，或达到轮数上限，命令会报告问题并以退出码 `1` 停止。已应用的更改会保留；再次运行前，请检查规则和设置。文件访问错误及其他工具错误也会终止命令。

您可以将 `--max-passes` 提高到 `256`。如果执行 `256` 轮后修复仍未稳定，请向 Mago 报告此缺陷。

## 退出码

| 码 | 含义 |
| :--- | :--- |
| `0` | 成功。`mago fix` 没有找到更多允许应用的修复。 |
| `1` | 有问题需要处理，或修复未能稳定。 |
| `2` | 工具错误:配置、I/O 等。 |
