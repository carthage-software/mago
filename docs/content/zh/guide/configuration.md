+++
title = "配置"
description = "Mago 如何发现配置文件、mago.toml 中各选项的作用,以及如何在多个项目间共享配置。"
nav_order = 40
nav_section = "指南"
+++
# 配置

Mago 从单一文件读取配置,通常是项目根目录下的 `mago.toml`。运行 `mago init` 可以生成一份骨架,也可以手写。

本页涵盖配置发现、`extends` 指令、全局选项以及 `[source]` 和 `[parser]` 小节。各工具特有的选项记录在每个工具的参考页面下。

## 发现

未传入 `--config` 时,Mago 按以下顺序查找配置文件:

1. 工作区目录(当前工作目录,或由 `--workspace` 指定的路径)。
2. 已设置时的 `$XDG_CONFIG_HOME`,例如 `$XDG_CONFIG_HOME/mago.toml`。
3. `$HOME/.config`,例如 `~/.config/mago.toml`。
4. `$HOME`,例如 `~/mago.toml`。

在每个位置,会先查找 `mago.{toml,yaml,yml,json}`,再查找 `mago.dist.{toml,yaml,yml,json}`。同一目录内的格式优先级是 `toml > yaml > yml > json`。第一个被找到的文件胜出,这让你可以为没有本地配置的项目在 `~/.config/mago.toml` 中保留一份全局配置。

## 用 `extends` 共享配置

> 自 Mago 1.25 起可用。更早的版本会静默忽略该指令。

`extends` 指令让一份配置可以叠加在其他配置之上,而不必复制粘贴。在多个项目共享同一份基线规范时很有用。

```toml
# Single parent
extends = "vendor/some-org/mago-config/mago.toml"

# Or a list, applied left-to-right; each later layer overrides earlier ones
extends = [
  "vendor/some-org/mago-config",     # directory: mago.{toml,yaml,yml,json} inside
  "configs/strict.json",              # mixing formats is fine
  "../shared/team-defaults.toml",
]

# This file's own keys override anything from the layers above
php-version = "8.3"
```

### 路径解析

绝对路径按原样使用。相对路径相对于声明 `extends` 的文件所在目录解析,而不是相对于当前工作目录。所以 `mago --config some/dir/config.toml` 配合 `extends = "base.toml"` 时,会查找 `some/dir/base.toml`。

文件类条目必须存在,且使用受支持的扩展名(`.toml`、`.yaml`、`.yml`、`.json`)。目录类条目会按上述优先级在其中扫描 `mago.{toml,yaml,yml,json}`;一个不含可识别文件的目录会被跳过并打印一条警告,而不是让构建失败。

### 实际优先级

各层按"后写入者覆盖、最深层先解析"的方式合并:

1. 内置默认值。
2. 每一个 `extends` 层,递归处理。父层自身的 `extends` 会先解析,再应用其键。
3. 拥有该 `extends` 的文件自身的键。
4. 针对 [受支持的标量](/guide/environment-variables/) 的 `MAGO_*` 环境变量。
5. CLI 参数,如 `--php-version`、`--threads`。

### 合并语义

按顶层键划分:

- 表(table)和对象进行深度合并。子层可以覆盖嵌套表中的某个键,而无需重定义整张表。
- 像 `source.excludes` 以及每条规则的 `exclude` 列表这类数组按顺序拼接,父层在前。如果基础配置排除了 `vendor/`,你保留这条排除并追加自己的。
- 标量(字符串、数字、布尔)由子层覆盖。

```toml
# base.toml
threads = 4
[source]
excludes = ["vendor", "node_modules"]
```

```toml
# project mago.toml
extends = "base.toml"
threads = 8
[source]
excludes = ["build"]   # appended -> ["vendor", "node_modules", "build"]
```

通过追踪规范化路径来检测环路,并以清晰的错误提示中断,而不是无限递归。菱形继承(A 继承 B 和 C,两者都继承 D)会只处理 D 一次,完全没有问题。各层可以自由混用格式;每一层由其对应的解析驱动负责解析,在通用值层面合并,最终文档再依据 schema 进行校验。

## 全局选项

这些键位于 `mago.toml` 的根层。

```toml
version = "1"
php-version = "8.2"
threads = 8
stack-size = 8388608     # 8 MiB
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `version` | string | 无 | 锁定本项目所基于的 Mago 版本。接受主版本(`"1"`)、次版本(`"1.25"`)或精确版本(`"1.25.1"`)的锁定。参见 [版本锁定](#version-pinning)。 |
| `php-version` | string | 最新稳定版 | Mago 在解析与分析时应针对的 PHP 版本。`mago init` 会尽可能从 `composer.json` 自动检测。 |
| `allow-unsupported-php-version` | boolean | `false` | 允许 Mago 运行在它官方不支持的 PHP 版本上。不建议使用。 |
| `no-version-check` | boolean | `false` | 在已安装二进制与锁定版本不一致时静默警告。主版本不一致始终是致命错误。 |
| `threads` | integer | 逻辑 CPU 数 | 用于并行工作的线程数。 |
| `stack-size` | integer | 2 MiB | 每线程栈大小,以字节为单位。最小 2 MiB,最大 8 MiB。 |
| `editor-url` | string | 无 | 终端输出中可点击文件路径的 URL 模板。参见 [编辑器集成](#editor-integration)。 |

### 版本锁定

锁定版本能让已安装二进制与项目期望之间的差异尽早暴露,而不是默默地输出不同结果。

三种锁定级别:

- **主版本锁定**(`version = "1"`):任何 `1.x.y` 都满足。升级到 `2.x` 会硬性报错,因为新主版本可能带来不兼容的默认值、schema 变化或规则行为。这是 `mago init` 默认写入的级别。
- **次版本锁定**(`version = "1.25"`):任何 `1.25.y` 都满足。漂移到不同的次版本会发出警告;跨主版本仍然是致命错误。
- **精确锁定**(`version = "1.25.1"`):任何漂移都会发出警告;跨主版本仍然是致命错误。

警告可通过 `--no-version-check`、`MAGO_NO_VERSION_CHECK` 环境变量,或配置中的 `no-version-check = true` 静默。这些都不会影响主版本漂移,而后者正是版本锁定的全部意义所在。

把已安装二进制同步到项目的锁定版本:

```sh
mago self-update --to-project-version
```

对于精确锁定,会直接解析到对应的发布 tag。对于主版本或次版本锁定,Mago 会扫描近期的 GitHub 发布,安装满足锁定的最高版本。所以即便 2.0 已经发布,`version = "1"` 仍会安装最新的 1.x。

`version` 目前是可选项。未来某个 Mago 版本可能在未设置时开始警告,以提醒项目为最终的 2.0 升级做准备。

## `[source]`

`[source]` 小节控制 Mago 如何发现和处理文件。

### 三类路径

Mago 区分你的代码、第三方代码,以及完全要忽略的代码:

- **`paths`** 是你的源码文件。Mago 会对它们进行分析、lint 和格式化。
- **`includes`** 是依赖(通常是 `vendor`)。Mago 会解析它们以解析符号和类型,但绝不会分析、lint 或重写它们。
- **`excludes`** 是 Mago 完全忽略的路径或 glob。它们对所有工具生效。

如果一个文件同时匹配 `paths` 和 `includes`,更具体的模式胜出。精确文件路径最具体,其次是更深的目录路径,然后是较浅的目录路径,最后是 glob 模式。当模式具体程度相同,`includes` 胜出,这让你可以显式地把某个路径标记为依赖。

```toml
[source]
paths     = ["src", "tests"]
includes  = ["vendor"]
excludes  = ["cache/**", "build/**", "var/**"]
extensions = ["php"]
```

三种列表都支持 glob 模式:

```toml
[source]
paths    = ["src/**/*.php"]
includes = ["vendor/symfony/**/*.php"]   # only Symfony from vendor
excludes = [
  "**/*_generated.php",
  "**/tests/**",
  "src/Legacy/**",
]
```

### 参考

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `paths` | string list | `[]` | 你源码的目录或 glob。为空时扫描整个工作区。 |
| `includes` | string list | `[]` | Mago 应解析但不修改的第三方代码的目录或 glob。 |
| `excludes` | string list | `[]` | 在所有工具中都被排除的 glob 或路径。 |
| `extensions` | string list | `["php"]` | 视为 PHP 的文件扩展名。 |

### Glob 设置

`[source.glob]` 用于调整 glob 的匹配方式。自 1.19 起可用。

```toml
[source.glob]
literal-separator = true     # `*` does not match `/`; use `**` for recursion
case-insensitive  = false
backslash-escape  = true     # `\` escapes special characters
empty-alternates  = false    # `{,a}` matches "" and "a" when true
```

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `case-insensitive` | bool | `false` | 模式匹配是否大小写不敏感。 |
| `literal-separator` | bool | `false` | 为 `true` 时,`*` 不匹配路径分隔符。用 `**` 实现递归匹配。 |
| `backslash-escape` | bool | `true`(Windows 上为 false) | `\` 是否用于转义特殊字符。 |
| `empty-alternates` | bool | `false` | 是否允许空备选项。 |

> `mago init` 生成的项目会设置 `literal-separator = true`。它让 `*` 的行为符合大多数用户的预期,即与 `.gitignore` 一样只匹配单层目录。

### 工具特定的排除

每个工具都有自己的可选 `excludes`。它们是叠加的:文件只要匹配全局列表或工具特定列表中的任意一项,就会被排除。

```toml
[source]
paths    = ["src", "tests"]
excludes = ["cache/**"]            # all tools

[analyzer]
excludes = ["tests/**/*.php"]      # only the analyzer

[formatter]
excludes = ["src/**/AutoGenerated/**/*.php"]

[linter]
excludes = ["database/migrations/**"]
```

linter 还支持按规则的路径排除,适用于让某条规则跳过某个路径而其他规则照常生效的场景。glob 模式在那里需要 Mago 1.20 或更高版本。完整参考见 [linter 配置页面](/tools/linter/configuration-reference/#per-rule-excludes)。

```toml
[linter.rules]
prefer-static-closure = { exclude = ["tests/"] }
no-global             = { exclude = ["**/*Test.php"] }
```

> 用 `mago list-files` 来确认 Mago 实际会处理的文件。`mago list-files --command formatter` 显示格式化器会触及的文件,`--command analyzer` 显示分析器的视角,以此类推。

## `[parser]`

```toml
[parser]
enable-short-tags = false
```

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `enable-short-tags` | boolean | `true` | 是否在 `<?php` 和 `<?=` 之外识别短开标签 `<?`。等同于 PHP 的 `short_open_tag` ini 指令。 |

当你的 `.php` 文件包含字面量 `<?xml` 声明,或并非真正 PHP 的模板片段时,可关闭短开标签。设置 `enable-short-tags = false` 后,像 `<?xml version="1.0"?>` 这样的序列会被当作内联文本而不是解析错误。代价是:任何依赖 `<?` 作为 PHP 开标签的代码不再被识别。

## 编辑器集成

Mago 可以把诊断输出中的文件路径渲染为 [OSC 8 超链接](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda)。在终端里点击路径,你的编辑器会在对应的行号和列号上打开文件。受支持的终端包括 iTerm2、WezTerm、Kitty、Windows Terminal、Ghostty 以及其他若干款。

Mago 会在可能的情况下自动检测正在运行的编辑器。在 macOS 上读取 `__CFBundleIdentifier`;其他平台上检查 `TERM_PROGRAM`。下列编辑器开箱即被识别:

- PhpStorm、IntelliJ IDEA、WebStorm
- VS Code、VS Code Insiders
- Zed
- Sublime Text

如果自动检测失败,可显式配置 URL。优先级遵循"先匹配先生效":

1. `MAGO_EDITOR_URL` 环境变量。
2. `mago.toml` 中的 `editor-url`。
3. 自动检测。

```sh
export MAGO_EDITOR_URL="vscode://file/%file%:%line%:%column%"
```

```toml
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

| 占位符 | 含义 |
| :--- | :--- |
| `%file%` | 文件的绝对路径。 |
| `%line%` | 行号,从 1 开始。 |
| `%column%` | 列号,从 1 开始。 |

常用模板:

| 编辑器 | 模板 |
| :--- | :--- |
| VS Code | `vscode://file/%file%:%line%:%column%` |
| VS Code Insiders | `vscode-insiders://file/%file%:%line%:%column%` |
| Cursor | `cursor://file/%file%:%line%:%column%` |
| Windsurf | `windsurf://file/%file%:%line%:%column%` |
| PhpStorm / IntelliJ | `phpstorm://open?file=%file%&line=%line%&column=%column%` |
| Zed | `zed://file/%file%:%line%:%column%` |
| Sublime Text | `subl://open?url=file://%file%&line=%line%&column=%column%` |
| Emacs | `emacs://open?url=file://%file%&line=%line%&column=%column%` |
| Atom | `atom://core/open/file?filename=%file%&line=%line%&column=%column%` |

> 只有在输出是启用了颜色的终端时,超链接才会被渲染。当输出被管道接收或设置了 `--colors=never` 时,超链接会被自动抑制,不会干扰脚本或 CI。

超链接在 `rich`(默认)、`medium`、`short` 和 `emacs` 报告格式中出现。机器可读格式(`json`、`github`、`gitlab`、`checkstyle`、`sarif`)不受影响。

## 工具特定的配置

每个工具都有自己的参考页面,涵盖该工具的选项:

- [Linter](/tools/linter/configuration-reference/)
- [格式化器](/tools/formatter/configuration-reference/)
- [分析器](/tools/analyzer/configuration-reference/)
- [Guard](/tools/guard/configuration-reference/)

## 检视合并后的配置

`mago config` 打印 Mago 实际使用的配置,即把默认值、每一层 `extends`、环境变量和 CLI 参数合并之后的结果。当某些行为不符合预期时很有用。

```sh
mago config                       # full config as pretty-printed JSON
mago config --show linter         # only the [linter] section
mago config --show formatter
mago config --default             # the built-in defaults
mago config --schema              # JSON Schema for the whole config
mago config --schema --show linter
```

| 参数 | 说明 |
| :--- | :--- |
| `--show <SECTION>` | 仅打印某一小节。可选值:`source`、`parser`、`linter`、`formatter`、`analyzer`、`guard`。 |
| `--default` | 打印内置默认值,而不是合并后的结果。 |
| `--schema` | 打印 JSON Schema,可用于 IDE 集成或外部工具。 |
| `-h`, `--help` | 打印帮助并退出。 |

全局参数必须放在 `config` 之前。完整列表见 [CLI 概览](/fundamentals/command-line-interface/)。
