+++
title = "Twig 模板"
description = "格式化 Twig 语法，不重新格式化 HTML。"
nav_order = 35
nav_section = "工具"
nav_subsection = "Formatter"
+++
# Twig 模板

`mago fmt` 支持 `.twig` 文件，包括 `index.html.twig`。它会在 `source.paths` 下查找这些文件，无需把 `twig` 加入 `source.extensions`。PHP 工具的文件选择不变。

```sh
mago fmt templates/
mago fmt --check
mago fmt --dry-run
mago fmt --staged
mago fmt --stdin-input --stdin-filepath templates/index.html.twig < templates/index.html.twig
```

从标准输入读取时，用 `--stdin-filepath` 选择 Twig。未提供路径时，Mago 使用 PHP 格式化器。`mago fix` 的格式化阶段也支持 Twig。

## 范围

Mago 格式化 Twig 标签、表达式、注释和缩进。空格遵循 [Twig 编码规范](https://twig.symfony.com/doc/3.x/coding_standards.html)。命名实参保留原有分隔符（`name: value` 或 `name = value`），避免格式化后需要更新 Twig 版本。宏参数的默认值仍使用 `name = value`。

Mago 不解析或重新格式化 HTML、CSS、JavaScript 或其他模板文本。字符串内容及其中的插值、空白控制标记（`-` 和 `~`）以及 `verbatim` 块的内容保持不变。未知扩展标签保持原样。如果空格变化会使 `not` 等名称变成运算符，该标签也保留原有布局。文件存在语法错误时，Mago 不修改该文件。

使用旧版 `raw` 块的文件也保持原样。

Mago 使用 Twig 的内置运算符。如果文件依赖扩展添加的自定义运算符，请使用忽略标记。

Mago 不添加文件末尾换行，也不更改模板文本中的换行符。`end-of-line` 仅控制格式化器在 Twig 标签内部添加的换行符。

格式化会改变 Twig 的 `source()` 函数读取到的文件内容。对于源文本必须保持不变的文件，请使用 `@mago-format-ignore`。

为限制栈空间占用，格式化时解析器的嵌套深度上限为 32 层。超过上限时返回错误，文件保持不变。

## 缩进

Twig 会将标签外的空白作为文本输出。Mago 保留这些文本的每一个字节，包括行首空白、空行、换行符和文件末尾换行。它只在标签内部拆分代码时添加缩进，新行沿用标签已有的缩进。

设置 `print-width = 30` 时：

```twig
  {{
      user.name|default(
          'A guest with a longer name'
      )
  }}
```

没有任何设置会改写标签外的空白。含有行内注释、含义不明确的关键字名称或过深表达式的标签会保持原样，以免改变行为。

## 配置

Twig 设置位于 `[formatter.twig]` 下，不继承 PHP 格式化器的设置或预设。`source.excludes` 和 `formatter.excludes` 同时适用于两种语言。

```toml
[formatter.twig]
enabled = true
print-width = 120
tab-width = 4
use-tabs = false
end-of-line = "auto"
```

| 选项 | 默认值 | 作用 |
| :--- | :--- | :--- |
| `enabled` | `true` | 格式化 `.twig` 文件。关闭后文件保持不变，标准输入和 Git 暂存文件也一样。 |
| `print-width` | `120` | Twig 表达式的目标行宽。不会拆分长字符串。 |
| `tab-width` | `4` | 每级缩进的空格数及制表符宽度。 |
| `use-tabs` | `false` | 使用制表符缩进。 |
| `end-of-line` | `"auto"` | 新增换行使用 `auto`、`lf`、`crlf` 或 `cr`。`auto` 使用 Twig 标签外模板文本中的首个换行符；没有时使用 LF。 |

关闭 Twig 格式化：

```toml
[formatter.twig]
enabled = false
```

## 忽略标记

可以在 Twig 注释中使用与 PHP 格式化器相同的标记：

```twig
{# @mago-format-ignore-next #}
{{  leave_this_alone  }}

{# @mago-format-ignore-start #}
{{  leave_this_region_alone  }}
{# @mago-format-ignore-end #}
```

`@mago-format-ignore` 跳过整个文件。`-next` 跳过下一条 Twig 语句；如果它开启一个块，则整个块都跳过。`@mago-formatter-ignore` 及其后缀同样有效。
