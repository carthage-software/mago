+++
title = "Twig Linter"
description = "使用 Mago 报告、pragma 和安全修复检查 Twig。"
nav_order = 40
nav_section = "Tools"
nav_subsection = "Twig Linter"
+++
# Twig Linter

`mago twig:lint` 检查 Twig 模板。扫描目录时，它选择 `source.paths` 下的 `.twig` 文件（包括 `index.html.twig`），不受 `source.extensions` 影响。显式文件、stdin 路径和替换文件始终按 Twig 处理，即使没有扩展名或使用 `.php`。此命令不会使用 PHP 解析器或规则。

目录扫描和 `--staged` 区分 `.twig` 扩展名的大小写。显式 `.TWIG` 路径仍可使用。

```sh
mago twig:lint templates/
mago twig:lint --only twig-no-dump
mago twig:lint --list-rules --json
mago twig:lint --explain twig-no-raw
mago twig:lint --stdin-input templates/page.html.twig < templates/page.html.twig
mago twig:lint templates/ --fix
```

`--semantics` 只检查 Twig 语法，不运行 lint 策略或类型分析。`--only` 使用带 `twig-` 前缀的代码，即使规则设置将其关闭，也会启用所选规则。但它不会绕过 `enabled = false`、路径排除或最低 Twig 版本。`--pedantic` 会启用所有受支持的规则，包括可选的项目策略。

命令使用 Mago 的报告格式、严重程度阈值、统计、基线和修复安全选项。`--staged` 选择 Git 已暂存的 `.twig` 文件；只有工作副本与暂存区一致时，才应用并暂存修复。

`--substitute ORIG=TEMP` 排除原文件，并把替换文件作为 Twig 检查。两个路径都必须为绝对路径。报告使用替换路径，其扩展名不限。`--stdin-input` 需要提供一个路径，用于报告和基线标识。

命令决定语言，`--only` 选择 Twig 规则。即使没有匹配的规则，Twig 语法检查仍会运行。`mago lint`、`mago analyze`、`mago fix` 和 `mago list-files` 保持各自原有行为，不会运行 Twig lint 规则。

## 配置

```toml
[twig.linter]
enabled = true
version = "3.30"
excludes = ["templates/generated/**"]

[twig.linter.rules.no-dump]
level = "error"
exclude = ["templates/debug/**"]

[twig.linter.rules.variable-name]
enabled = true

[twig.linter.rules.disallowed-functions]
enabled = true
names = ["source"]
```

| 选项 | 默认值 | 含义 |
| :--- | :--- | :--- |
| `enabled` | `true` | 运行 `twig:lint`。关闭后不扫描或检查文件。其他工具不变。 |
| `version` | `"3.0"` | 项目支持的最早 Twig 3 版本，格式为 `3.MINOR`。设置后才会启用特定版本的规则和修复。Mago 不从 Composer 推断此值，也不会据此检查全部语言功能。解析器接受当前 Twig 语法。 |
| `excludes` | `[]` | 仅针对 Twig 的路径前缀或 glob。在解析前排除文件。 |
| `baseline` | 无 | 相对 `source.workspace` 的基线路径，也可使用绝对路径。`--baseline` 可覆盖此值。 |
| `baseline-variant` | `"loose"` | 写基线时使用按数量匹配的 `loose` 或按行匹配的 `strict`。 |
| `minimum-fail-level` | `"error"` | 使命令失败的最低级别，可由 `--minimum-fail-level` 覆盖。 |
| `rules.<名称>.enabled` | 取决于规则 | 开关规则。配置键省略 `twig-` 前缀。 |
| `rules.<名称>.level` | 取决于规则 | `note`、`help`、`warning` 或 `error`。`twig.linter.minimum-fail-level` 控制命令失败阈值。 |
| `rules.<名称>.exclude` | `[]` | 单条规则的路径前缀或 glob。使用 `source.glob` 设置及相对工作区的文件名。 |
| `rules.disallowed-functions.names` | `[]` | 禁用的精确函数名，包括导入的宏别名。还需启用规则。 |
| `rules.disallowed-filters.names` | `[]` | 禁用的精确过滤器名。还需启用规则。 |
| `rules.disallowed-tags.names` | `[]` | 禁用的精确标签名，包括扩展标签。还需启用规则。 |

`source.paths`、`source.excludes` 和 `source.glob` 是共用的文件设置。PHP `[linter]` 的规则、排除、基线及失败阈值不会影响 `twig:lint`。配置层按现有 `extends` 规则合并 `[twig.linter]`。未知字段和规则名会报错。可用 `mago config --show twig.linter` 或 `mago config --schema --show twig.linter` 查看配置。`config --show linter` 仍只显示 PHP 设置。

`mago config --show twig` 显示已配置的 Twig 工具。运行时，省略的工具使用各自的默认值；`mago config --show twig.linter --default` 显示 linter 的默认设置。

## 规则

[规则页面](/tools/twig-linter/rules/)列出每条规则的默认级别、所需 Twig 版本、设置和示例。规则代码以 `twig-` 开头；配置键省略该前缀。默认关闭的规则是可选策略，并不意味着有效 Twig 语法存在错误。

内置函数检查遵守宏导入别名及其词法作用域。未知扩展标签、过滤器、函数或测试不会仅因未知而报错。显式禁用名称的规则也会匹配宏别名。命名规则不改动非 ASCII 名称。动态键、字符串的一元转换和超出范围的数字转换保持未知。

设计参考了 [Twig 官方编码规范](https://twig.symfony.com/doc/3.x/coding_standards.html)、[Twig-CS-Fixer 规则目录](https://twigcsfixer.github.io/rules)和 [Twig 源码](https://github.com/twigphp/Twig)。空格、缩进、逗号等布局工作由 [Twig 格式化器](/tools/formatter/twig/)负责。Mago 不照搬可能更改渲染输出的文本或文件结尾修复。

## Pragma

独占一行的 `{# #}` pragma 与目标语句之间只能有空白或其他注释。渲染文本或结束标签会终止其作用域，pragma 不会抑制更后面的语句。

代码标签内独占一行的 `#` pragma 只覆盖该标签的剩余部分，不会延伸到 `if` 或其他块的主体，也不会影响后续标签。

```twig
{# @mago-expect lint:twig-no-dump(2) 此分支有意输出调试信息。 #}
{% if debug %}
    {{ dump(first) }}
    {{ dump(second) }}
{% endif %}
```

支持 `lint` 和 `linter` 类别、逗号分隔的代码以及重复类别前缀。语句前独占一行的注释覆盖该语句及其主体。行尾注释只覆盖所在行。更内层的注释优先。显式 `(N)` 限制匹配次数；省略次数时，作用域 pragma 保持现有的作用域内不限次数行为。`--only` 不会对非活动代码报告未满足的预期。完整检查中，关闭或排除的规则仍可能留下未满足的预期，与 PHP 行为一致。

Mago 会报告未使用的 ignore 和未满足的 expect。安全清理只修改注释主体，保留分隔符、空白控制标记和显式 `(1)`。分组清理可能需要再次运行 `twig:lint --fix`。`mago fix` 不运行 Twig lint 规则。

## 修复安全性与限制

安全修复仅更改命名实参分隔符、缺失的结束名称、保守范围内的字符串引号或 pragma 主体。它们不会重写渲染空白、字符串内容或求值顺序。命名实参分隔符修复要求显式设置最低版本为 3.12 或更高版本。目前 `--potentially-unsafe` 和 `--unsafe` 均不增加 Twig 修复。

`twig:lint` 会在扫描或写入之前拒绝 `--format-after-fix`。Twig 格式化必须单独运行；此命令不改变现有格式化器的语言选择。

含旧式 `raw` 块的文件仍会收到诊断，但在任何安全级别下都不会改动，包括 pragma 主体。旧版或自定义 `raw` 处理方式可能改变 Twig 对块边界的读取。`--semantics` 不读取 linter pragma。

Mago 假定使用 Twig 内置的词法分隔符、插值语法和运算符。自定义 lexer 改变其含义时，请排除相关文件或规则。未知标签参数和 `verbatim` 主体保持不透明。Mago 不知道应用是否替换了内置函数/过滤器或使用自定义调用解析器；请按项目调整规则。

任何源码修改都会改变 Twig `source()` 的返回值。如果输出契约要求源码字节不变，请排除对应文件。

解析器的嵌套预算为 32 级，一个结构可能占用多级。`twig-depth-limit` 表示 Mago 未检查该模板，不代表 Twig 拒绝它。其他解析错误使用 `twig-syntax-error`，此类模板不会运行任何规则。Pragma 无法屏蔽这两个代码。很长的平坦表达式链仍会被检查。debug 库路径已在 4 MiB 栈上测试；更小的调用方栈可能不足。Mago 不会通过增加 worker 栈大小来掩盖深层输入失败。
