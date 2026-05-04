+++
title = "Linter 配置参考"
description = "Mago 在 [linter] 与 [linter.rules] 下接受的所有选项。"
nav_order = 60
nav_section = "工具"
nav_subsection = "Linter"
+++
# 配置参考

linter 在 `mago.toml` 中由两个表配置:`[linter]` 用于工具级设置,`[linter.rules]` 用于逐规则的设置。

```toml
[linter]
integrations = ["symfony", "phpunit"]
excludes = ["src/Generated/"]
baseline = "linter-baseline.toml"

[linter.rules]
# Disable a rule completely
ambiguous-function-call = { enabled = false }

# Change a rule's severity level
no-else-clause = { level = "warning" }

# Configure a rule's specific options
cyclomatic-complexity = { threshold = 20 }

# Exclude specific paths from one rule
prefer-static-closure = { exclude = ["tests/"] }
```

## `[linter]`

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `excludes` | 字符串列表 | `[]` | linter 跳过的路径或 glob。叠加在全局 `source.excludes` 之上。 |
| `integrations` | 字符串列表 | `[]` | 要启用的框架集成。完整列表见[集成页面](/tools/linter/integrations/)。 |
| `baseline` | 字符串 | 无 | baseline 文件路径。设置后,linter 会将其作为默认 baseline,你不必每次都传 `--baseline`。CLI 的 `--baseline` 会覆盖该值。 |
| `baseline-variant` | 字符串 | `"loose"` | 新生成的 baseline 所用的变体。可选 `"loose"`(基于计数,具备容错性)或 `"strict"`(精确到行)。参见 [baseline 变体](/fundamentals/baseline/#two-variants)。 |
| `minimum-fail-level` | 字符串 | `"error"` | 触发非零退出码的最低严重等级。取值:`"note"`、`"help"`、`"warning"`、`"error"`。CLI 的 `--minimum-fail-level` 会覆盖该值。 |

这里的 `excludes` 是在全局列表之上叠加。被全局匹配到的文件总是会被排除;此选项只能让你额外为 linter 排除一些文件。

```toml
[source]
excludes = ["cache/**"]              # 对所有工具排除

[linter]
excludes = ["database/migrations/**"]  # 额外对 linter 排除
```

## `[linter.rules]`

此表下的每个键都是一条规则的代码,以 `kebab-case` 书写。每条规则都接受下面的通用选项;部分规则还接受自身的选项。

### 通用选项

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `enabled` | 布尔值 | 视规则而定 | 启用或禁用该规则。 |
| `level` | 字符串 | 视规则而定 | 严重等级。取值:`"error"`、`"warning"`、`"help"`、`"note"`。 |
| `exclude` | 字符串列表 | `[]` | 该规则跳过的路径或 glob。其他规则仍会作用于这些文件。 |

### 逐规则的 excludes

当一条规则总体上有价值,但不适用于代码库的某一部分(例如生成代码或测试夹具)时,`exclude` 就很有用。

```toml
[linter.rules]
prefer-static-closure = { enabled = true, exclude = ["tests/"] }
no-goto              = { exclude = ["src/Legacy/"] }
no-eval              = { exclude = ["src/Templating/Compiler.php"] }
no-global            = { exclude = ["**/*Test.php"] }
```

每个条目可以是普通路径或 glob:

- 普通路径(`"tests"`、`"tests/"`、`"src/Foo.php"`)以前缀匹配的方式针对项目根目录的相对路径进行匹配。
- glob 模式(任何包含 `*`、`?`、`[` 或 `{` 的条目)使用与全局 `source.excludes` 相同的 glob 引擎,匹配完整的相对路径,并应用 `[source.glob]` 设置。

逐规则 `exclude` 中使用 glob 模式需要 Mago 1.20 或更高版本。早期版本仅接受普通前缀路径。

逐规则 `exclude` 与 `[linter].excludes` 不同:

- `[linter].excludes` 会从所有规则中移除文件。
- 规则自身的 `exclude` 只会从这一条规则中移除文件,其他规则仍然作用于该文件。

### 规则专属选项

部分规则接受额外选项。`cyclomatic-complexity` 是一个典型例子:

```toml
[linter.rules]
cyclomatic-complexity = { level = "error", threshold = 15 }
```

要查看某条规则的专属选项,可询问 Mago:

```sh
mago lint --explain cyclomatic-complexity
```

完整的逐规则参考见[规则页面](/tools/linter/rules/)。
