+++
title = "Guard 配置参考"
description = "Mago 在 [guard] 下接受的所有选项。"
nav_order = 40
nav_section = "工具"
nav_subsection = "Guard"
+++
# 配置参考

设置位于 `mago.toml` 的 `[guard]` 下。配置由两部分组成:`[guard.perimeter]` 用于依赖规则,`[[guard.structural.rules]]` 用于结构约定。

## 顶级选项

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `mode` | `string` | `"default"` | 运行哪部分检查。取值为 `"default"`、`"structural"`、`"perimeter"` 之一。 |
| `excludes` | `string[]` | `[]` | 要从分析中排除的路径或 glob 模式。叠加在 `[source].excludes` 之上。 |
| `baseline` | `string` | 未设置 | baseline 文件路径。等同于每次运行都传入 `--baseline`。 |
| `baseline-variant` | `string` | `"loose"` | 新生成的 baseline 所用格式。`"loose"`(基于计数)或 `"strict"`(精确到行)。参见 [baseline](/fundamentals/baseline/)。 |
| `minimum-fail-level` | `string` | `"error"` | 触发非零退出码的最低严重等级。取值为 `"note"`、`"help"`、`"warning"`、`"error"` 之一。会被 `--minimum-fail-level` 覆盖。 |

`mode` 控制 guard 的哪部分运行:

- `"default"` 同时运行两部分。
- `"structural"` 只运行结构检查。
- `"perimeter"` 只运行边界检查。

```toml
[guard]
mode = "structural"
```

`--structural` 与 `--perimeter` 标志会覆盖配置中的 mode。参见[命令参考](/tools/guard/command-reference/)。

这里的 `excludes` 是在你在 `[source].excludes` 中所设置的基础上叠加,绝不会缩小全局列表。

```toml
[source]
excludes = ["cache/**"]

[guard]
excludes = ["src/ThirdParty/**"]
```

## 边界 guard

边界部分定义项目各部分之间的依赖规则。

```toml
[guard.perimeter]
layering = [
    "CarthageSoftware\\Domain",
    "CarthageSoftware\\Application",
    "CarthageSoftware\\UI",
    "CarthageSoftware\\Infrastructure",
]

[guard.perimeter.layers]
core = ["@native", "Psl\\**"]
psr = ["Psr\\**"]
framework = ["Symfony\\**", "Doctrine\\**"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Domain"
permit = ["@layer:core"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Application"
permit = ["@layer:core", "@layer:psr"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Infrastructure"
permit = ["@layer:core", "@layer:psr", "@layer:framework"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Tests"
permit = ["@all"]
```

### `layering`

一个有序的命名空间列表,从最独立的核心,依次向外延展到最外层。每一层只能依赖在其之前定义的层。指向更外层的依赖会触发违规。

### 层别名

`[guard.perimeter.layers]` 定义可重用的命名空间和路径分组,在规则中通过 `@layer:<name>` 引用。

### 规则

每个 `[[guard.perimeter.rules]]` 表定义一条规则:

- `namespace`:此规则所适用的命名空间。可以是以 `\` 结尾的命名空间,或特殊关键字 `@global`(表示全局命名空间)。
- `permit`:被允许的依赖。可以是字符串列表或带详细字段的对象列表。

#### `permit` 的取值

`permit` 接受路径。路径可以是关键字、命名空间、符号或 glob 模式。

| 路径 | 说明 |
| :--- | :--- |
| `@global` | 全局命名空间中定义的符号。 |
| `@all` | 项目中任何位置的任意符号,包括 vendor 包。对测试很有用。 |
| `@self` / `@this` | 与规则的 `namespace` 同一根命名空间内的任意符号。 |
| `@native` / `@php` | PHP 内置的函数、类和常量。 |
| `@layer:<name>` | `[guard.perimeter.layers]` 中具名别名所包含的所有命名空间和路径。 |
| `App\Shared\\**` | glob 模式。`*` 匹配单个命名空间段,`**` 匹配零个或多个。 |
| `App\Service` | 精确的完全限定符号名。 |
| `App\Service\\` | 精确的命名空间。允许其中直接定义的符号。 |

你可以使用对象形式按符号种类来收窄某条权限:

```toml
[[guard.perimeter.rules]]
namespace = "DoctrineMigrations\\"
permit = [{ path = "@all", kinds = ["class-like"] }]
```

- `path`:上面任意一种路径形式。
- `kinds`:被允许的符号种类。取值为 `class-like`(涵盖类、接口、trait、枚举)、`function`、`constant`、`attribute`。

## 结构 guard

`[[guard.structural.rules]]` 定义结构约定。每个条目把用于挑选符号的选择器,与所选符号必须满足的约束组合在一起。

```toml
[[guard.structural.rules]]
on = "CarthageSoftware\\UI\\**\\Controller\\**"
target = "class"
must-be-named = "*Controller"
must-be-final = true
must-be-readonly = true
reason = "Controllers must be final and follow naming conventions."

[[guard.structural.rules]]
on = "CarthageSoftware\\Domain\\**\\Repository\\**"
target = "interface"
must-be-named = "*RepositoryInterface"
reason = "Domain repository interfaces must follow a standard naming convention."

[[guard.structural.rules]]
on = "CarthageSoftware\\Infrastructure\\**\\Repository\\**"
target = "class"
must-be-final = true
must-extend = "CarthageSoftware\\Infrastructure\\Shared\\Repository\\AbstractRepository"
reason = "Infrastructure repositories must extend our abstract class."

[[guard.structural.rules]]
on = "CarthageSoftware\\Domain\\**\\Enum\\**"
must-be = ["enum"]
reason = "This namespace is designated for enums only."
```

### 选择器

| 键 | 说明 |
| :--- | :--- |
| `on` | 必填。glob 模式,匹配此规则适用的符号的完全限定名。 |
| `not-on` | 可选 glob 模式,排除原本会被 `on` 匹配到的符号。 |
| `target` | 可选过滤器,把规则限定到一种符号种类。取值为 `class`、`interface`、`trait`、`enum`、`function`、`constant` 之一。 |

### 约束

| 键 | 说明 |
| :--- | :--- |
| `must-be` | 把所选命名空间限定为只能包含所列符号种类。取值:`class`、`interface`、`trait`、`enum`、`function`、`constant`。 |
| `must-be-named` | 符号名必须匹配的 glob 模式(例如 `*Controller`)。 |
| `must-be-final` | 布尔值。`true` 要求 `final`;`false` 禁止 `final`。 |
| `must-be-abstract` | 布尔值。`true` 要求 `abstract`;`false` 禁止 `abstract`。 |
| `must-be-readonly` | 布尔值。`true` 要求 `readonly`;`false` 禁止 `readonly`。 |
| `must-implement` | 类必须实现的一个或多个接口。 |
| `must-extend` | 符号必须继承的一个类。 |
| `must-use-trait` | 符号必须使用的一个或多个 trait。 |
| `must-use-attribute` | 符号必须携带的一个或多个 attribute。 |
| `reason` | 显示在错误消息中的人类可读说明。 |

#### 继承约束的形态

`must-implement`、`must-extend`、`must-use-trait` 和 `must-use-attribute` 接受单个字符串、字符串数组(AND)或字符串数组的数组(AND 的 OR)。字面量 `"@nothing"` 表示禁止任何取值。

```toml
must-extend = "App\\BaseClass"

must-implement = ["App\\InterfaceA", "App\\InterfaceB"]

must-extend = [
    ["App\\AbstractA", "App\\AbstractB"],
    ["App\\AbstractC"],
]

must-implement = "@nothing"
```
