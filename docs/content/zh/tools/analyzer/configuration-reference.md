+++
title = "分析器配置参考"
description = "Mago 在 [analyzer] 下接受的所有选项。"
nav_order = 30
nav_section = "工具"
nav_subsection = "Analyzer"
+++
# 配置参考

设置位于 `mago.toml` 的 `[analyzer]` 下。

```toml
[analyzer]
ignore = ["mixed-argument"]
baseline = "analyzer-baseline.toml"
```

## 通用选项

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `excludes` | `string[]` | `[]` | 要从分析中排除的路径或 glob 模式。叠加在 `[source].excludes` 之上。 |
| `ignore` | `(string \| object)[]` | `[]` | 要忽略的问题代码,可选地限定到特定路径。详见下文。 |
| `baseline` | `string` | 未设置 | baseline 文件路径。等同于每次运行都传入 `--baseline`。CLI 标志会覆盖该值。 |
| `baseline-variant` | `string` | `"loose"` | 新生成的 baseline 所用格式。`"loose"`(基于计数)或 `"strict"`(精确到行)。参见 [baseline](/fundamentals/baseline/)。 |
| `minimum-fail-level` | `string` | `"error"` | 触发非零退出码的最低严重等级。取值为 `"note"`、`"help"`、`"warning"`、`"error"` 之一。会被 `--minimum-fail-level` 覆盖。 |

这里的 `excludes` 是在你在 `[source].excludes` 中所设置的基础上叠加,绝不会缩小全局列表。

```toml
[source]
excludes = ["cache/**"]

[analyzer]
excludes = ["tests/**/*.php"]
```

### 路径限定的忽略

`ignore` 接受普通字符串、单路径对象和多路径对象,可自由混用:

```toml
[analyzer]
ignore = [
    "mixed-argument",
    { code = "missing-return-type", in = "tests/" },
    { code = "unused-parameter", in = ["tests/", "src/Generated/"] },
]
```

`in` 中的每个条目要么是一个目录或文件前缀,要么是一个 glob 模式。任何包含 `*`、`?`、`[` 或 `{` 的值会被视为 glob,匹配完整的相对路径;其他值按前缀匹配。`"tests"` 和 `"tests/"` 都能匹配 `tests` 下的每个文件。

```toml
[analyzer]
ignore = [
    { code = "mixed-assignment", in = [
        "tests/",
        "src/Legacy/**/*.php",
        "modules/*/Generated/*.php",
    ] },
]
```

glob 匹配会遵循 `[source.glob]` 下的项目级设置,因此像 `literal-separator` 和 `case-insensitive` 这样的开关在这里同样适用。

`excludes` 与 `ignore` 含义不同。`excludes` 把文件完全从分析中移除,因此这些文件不会被解析以提供类型信息。`ignore` 仍会分析文件,但会在输出中抑制所列代码。

## 特性开关

这些开关用于切换各项独立的分析。默认值已为日常使用调好;在代码库逐步收紧时再开启它们。

| 选项 | 默认值 | 说明 |
| :--- | :--- | :--- |
| `find-unused-expressions` | `true` | 报告其结果被丢弃的表达式,例如 `$a + $b;`。 |
| `find-unused-definitions` | `true` | 报告从未被引用的私有定义。 |
| `find-overly-wide-return-types` | `false` | 当声明的返回类型包含主体永远不会产生的分支时发出警告,例如总是返回字符串的函数声明为 `: string\|false`。自 1.20.0 起可用。 |
| `analyze-dead-code` | `false` | 分析看似不可达的代码。 |
| `memoize-properties` | `true` | 跟踪字面量属性值以获得更精准的推断,代价是占用一些额外内存。 |
| `allow-possibly-undefined-array-keys` | `true` | 允许访问可能不存在的键,而不报告。 |
| `check-throws` | `false` | 报告未被捕获且未通过 `@throws` 声明的异常。 |
| `check-missing-override` | `false` | 报告重写方法上缺失的 `#[Override]` attribute(PHP 8.3+)。 |
| `find-unused-parameters` | `false` | 报告从未被读取的参数。 |
| `strict-list-index-checks` | `false` | 要求作为列表下标使用的整数必须可被证明非负。 |
| `no-boolean-literal-comparison` | `false` | 禁止与布尔字面量进行直接比较,例如 `$a === true`。 |
| `check-missing-type-hints` | `false` | 报告参数、属性和返回值缺失的类型提示。 |
| `check-closure-missing-type-hints` | `false` | 把类型提示检查扩展到闭包(需要 `check-missing-type-hints`)。 |
| `check-arrow-function-missing-type-hints` | `false` | 把类型提示检查扩展到箭头函数(需要 `check-missing-type-hints`)。 |
| `allow-implicit-pipe-callable-types` | `false` | 当可调用对象作为 `\|>` 的右操作数时,跳过对闭包/箭头函数的类型提示检查。 |
| `register-super-globals` | `true` | 自动注册 PHP 超全局变量,如 `$_GET` 与 `$_POST`。 |
| `trust-existence-checks` | `true` | 基于 `method_exists()`、`property_exists()`、`function_exists()` 与 `defined()` 收窄类型。 |
| `check-property-initialization` | `false` | 验证已注解类型的属性是否在构造函数或类初始化器中被初始化。 |
| `check-use-statements` | `false` | 报告导入了不存在的类、函数或常量的 use 语句。 |
| `check-name-casing` | `false` | 报告引用类、函数等时的大小写错误。有助于防止在区分大小写的文件系统上自动加载失败。 |
| `enforce-class-finality` | `false` | 报告未声明为 `final` 或 `abstract`、未注解 `@api`、且没有子类的类。 |
| `require-api-or-internal` | `false` | 要求抽象类、接口和 trait 必须以 `@api` 或 `@internal` 注解。 |
| `check-experimental` | `false` | 报告在非实验性上下文中使用 `@experimental` 符号的情况。自 1.19.0 起可用。 |
| `allow-side-effects-in-conditions` | `true` | 设为 `false` 时,报告在 `if`、`while`、`for`、三元或 `match` 条件中调用非纯函数的情况。 |

## 属性初始化

启用 `check-property-initialization` 时,分析器会报告两类问题:

- `missing-constructor`,针对带类型属性但没有构造函数的类。
- `uninitialized-property`,针对在构造函数中未被赋值的带类型属性。

`class-initializers` 让你可以将更多方法标记为初始化器,与 `__construct` 一同视作初始化方法。在这些方法中赋值的属性会被视为肯定已初始化。这对于使用生命周期方法的框架很有用。

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `class-initializers` | `string[]` | `[]` | 视作类初始化器的方法名。 |

```toml
[analyzer]
check-property-initialization = true
class-initializers = ["setUp", "initialize", "boot"]
```

在该配置下,以下代码不会触发误报:

```php
class MyTest extends TestCase
{
    private string $name;

    protected function setUp(): void
    {
        $this->name = "test";
    }
}
```

## 异常过滤

启用 `check-throws` 时,有两个选项可以让你跳过特定异常。

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `unchecked-exceptions` | `string[]` | `[]` | 要忽略的异常,包括其所有子类(感知继承层级)。 |
| `unchecked-exception-classes` | `string[]` | `[]` | 仅按精确类匹配忽略的异常。其子类和父类仍会被检查。 |

```toml
[analyzer]
check-throws = true

unchecked-exceptions = [
    "LogicException",
    "Psl\\Type\\Exception\\ExceptionInterface",
]

unchecked-exception-classes = [
    "Psl\\File\\Exception\\FileNotFoundException",
]
```

使用 `unchecked-exceptions` 来一次性屏蔽某一整类异常,例如所有 `LogicException` 的子类。当你想忽略某一个具体异常但仍跟踪其同级和父类时,使用 `unchecked-exception-classes`。

## 实验性 API 检测

设置 `check-experimental = true` 来标记从非实验性代码中使用 `@experimental` 符号的情况。用 PHPDoc 标签标注符号:

```php
/** @experimental */
class UnstableApi {}

/** @experimental */
function beta_feature(): void {}
```

当它们被稳定代码使用时,分析器会发出警告:

```php
new UnstableApi();              // warning
beta_feature();                 // warning
class MyService extends UnstableApi {}  // warning
```

在另一个实验性上下文中使用是允许的:

```php
/** @experimental */
function also_experimental(): void {
    new UnstableApi();
    beta_feature();
}

class StableService {
    /** @experimental */
    public function experimentalMethod(): void {
        new UnstableApi();
    }
}
```

## 插件

插件为各种库和框架提供类型提供器,使函数返回精确类型而非泛化类型。

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `disable-default-plugins` | `bool` | `false` | 禁用所有默认插件。仅 `plugins` 中列出的名称会激活。 |
| `plugins` | `string[]` | `[]` | 按名称或别名启用的插件。 |

### 可用插件

| 插件 | 别名 | 默认 | 说明 |
| :--- | :--- | :--- | :--- |
| `stdlib` | `standard`、`std`、`php-stdlib` | 已启用 | PHP 内置函数:`strlen`、`array_*`、`json_*` 等。 |
| `psl` | `php-standard-library`、`azjezz-psl` | 未启用 | [php-standard-library](https://github.com/php-standard-library/php-standard-library)。 |
| `flow-php` | `flow`、`flow-etl` | 未启用 | [flow-php/etl](https://github.com/flow-php/etl)。 |
| `psr-container` | `psr-11` | 未启用 | [psr/container](https://github.com/php-fig/container)。 |

举例来说,`stdlib` 插件会让分析器知道 `strlen($s)` 返回 `int<0, max>`,`json_decode($json, true)` 返回 `array<string, mixed>`,而 `array_filter($array)` 会保留输入形状但可能丢弃元素。

### 示例

使用默认值(仅 `stdlib`):

```toml
[analyzer]
```

启用更多插件:

```toml
[analyzer]
plugins = ["psl", "flow-php", "psr-container"]
```

全部禁用:

```toml
[analyzer]
disable-default-plugins = true
```

仅使用一个插件:

```toml
[analyzer]
disable-default-plugins = true
plugins = ["psl"]
```

插件别名在所有位置都生效,因此 `plugins = ["std"]` 与 `plugins = ["stdlib"]` 等价。

## 严格模式

分析器默认运行在适中的严格度。开启更多检查会更严格;对遗留代码可适度放宽。

### 最高严格度

```toml
[analyzer]
find-unused-expressions = true
find-unused-definitions = true
find-overly-wide-return-types = true
analyze-dead-code = true
check-throws = true
check-missing-override = true
find-unused-parameters = true
check-missing-type-hints = true
check-closure-missing-type-hints = true
check-arrow-function-missing-type-hints = true
enforce-class-finality = true
require-api-or-internal = true
check-experimental = true

strict-list-index-checks = true
no-boolean-literal-comparison = true

allow-possibly-undefined-array-keys = false
trust-existence-checks = false
```

### 宽松模式

```toml
[analyzer]
check-missing-type-hints = false
strict-list-index-checks = false
no-boolean-literal-comparison = false
enforce-class-finality = false
require-api-or-internal = false

allow-possibly-undefined-array-keys = true
trust-existence-checks = true

check-throws = false
```

把 Mago 引入既有代码库时,先以宽松模式起步,配合 [baseline](/fundamentals/baseline/),随着代码改善再逐步收紧。

### 关于个别开关的说明

`trust-existence-checks` 决定分析器是否基于运行时检查进行类型收窄。开启时(默认),下面的代码没问题:

```php
function process(object $obj): mixed
{
    if (method_exists($obj, 'toArray')) {
        return $obj->toArray();
    }

    return null;
}
```

关闭后,该调用就需要显式的类型保证。

`allow-implicit-pipe-callable-types` 在可调用对象作为 `|>` 的右操作数时跳过闭包/箭头函数的类型提示检查。管道的左操作数已携带足以推导参数的类型信息,因此那里缺失的类型提示是无害的。

## 性能调优

分析器使用内部阈值在分析深度与速度之间取得平衡。设置位于 `[analyzer.performance]` 下。

| 选项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `saturation-complexity-threshold` | `u16` | `8192` | CNF 饱和过程中的最大子句数。 |
| `disjunction-complexity-threshold` | `u16` | `4096` | OR 操作中每一侧的最大子句数。 |
| `negation-complexity-threshold` | `u16` | `4096` | 公式取反时的最大累计复杂度。 |
| `consensus-limit-threshold` | `u16` | `256` | consensus 优化遍数的上限。 |
| `formula-size-threshold` | `u16` | `512` | 跳过简化前的最大逻辑公式大小。 |
| `string-combination-threshold` | `u16` | `128` | 在泛化为 `string` 之前能跟踪的最大字面字符串数。 |
| `integer-combination-threshold` | `u16` | `128` | 在泛化为 `int` 之前能跟踪的最大字面整数数。 |
| `array-combination-threshold` | `u16` | `32` | 在合并之前能单独跟踪的最大封闭键控数组形状数。 |
| `loop-assignment-depth-threshold` | `u8` | `1` | 循环不动点迭代的最大深度。`0` 表示禁用重新迭代。 |

`string-concat-combination-threshold` 仍作为 `string-combination-threshold` 的别名被接受。

### 何时调整

默认值适合大多数项目。若分析感觉太慢,可调低这些阈值,代价是部分推断精度。在条件高度复杂的代码中需要更深入的推断时,可调高这些阈值。

### 每个阈值控制什么

分析器会把类型约束转换成合取范式(CNF)的逻辑公式。这些公式会随着复杂条件呈指数级增长,因此使用阈值防止计算失控。

- 饱和复杂度限制公式简化过程中处理的子句数。一旦超过,简化会提前停止。
- 析取复杂度限制 `OR` 合并时子句的增长。宽联合和大量分支可能触发此上限。
- 否定复杂度限制公式取反时的展开,例如从复杂 `if` 计算 `else` 分支。
- consensus 上限限制一种用于检测逻辑恒真式的优化遍数。值越高,可能发现更多简化机会。
- 公式大小是分析器回退到更简单推断之前的整体复杂度上限。
- 字符串/整数合并上限限制在分析器扩展到 `string` 或 `int` 之前可跟踪的字面值数量。没有它,非常大的数组或 switch 语句会把合并代价推到 O(n²)。
- 数组合并上限限制在类型合并过程中保持分离的封闭键控数组形状数。当过程式代码在不同分支上为同一变量积累大量略有差异的形状时,合并器会一直保留每一种,直到达到此阈值,然后将它们合并为一个泛化形状。对依赖非常精确的逐键收窄的代码,可调高它。 
- 循环赋值深度上限限制循环分析器对每个循环主体运行多少次不动点迭代。当存在 `N` 个跨循环依赖链时,可能需要多达 `N` 次额外遍历,链末端的类型才能完全稳定;每次遍历都会重新分析整个主体。默认值 `1` 对几乎所有真实代码都足够。把它调到 `2` 或 `3` 适合需要对深度跨循环依赖链做非常精确收窄的代码库。`0` 表示完全禁用重新迭代,这是最快的设置,但可能让一些自依赖类型比所需更宽。

### 示例

快速分析,精度较低:

```toml
[analyzer.performance]
saturation-complexity-threshold = 2048
disjunction-complexity-threshold = 1024
negation-complexity-threshold = 1024
consensus-limit-threshold = 64
formula-size-threshold = 128
string-combination-threshold = 64
integer-combination-threshold = 64
array-combination-threshold = 16
loop-assignment-depth-threshold = 1
```

深度分析,速度较慢:

```toml
[analyzer.performance]
saturation-complexity-threshold = 16384
disjunction-complexity-threshold = 8192
negation-complexity-threshold = 8192
consensus-limit-threshold = 512
formula-size-threshold = 1024
string-combination-threshold = 256
integer-combination-threshold = 256
array-combination-threshold = 256
loop-assignment-depth-threshold = 4
```

在条件逻辑沉重或包含数千次数组操作的代码库上,提高阈值可能会显著拉长分析时间。在部署到 CI 之前先在你的项目上测试。

### 诊断慢速运行

如果 Mago 感觉慢,通常预示 Mago 中存在 bug,而不是正常表现。作为参考,在 Apple M1 Pro 上,分析器对 [`WordPress/wordpress-develop`](https://github.com/WordPress/wordpress-develop) 全量分析耗时不到两秒,对 [`php-standard-library/php-standard-library`](https://github.com/php-standard-library/php-standard-library) 不到 200 毫秒。具体数字会随硬件和项目规模而变,但作为粗略阈值:如果 Mago 分析你的项目超过 30 秒,那就有问题,要么在 Mago 中,要么在它撞上了某种病态输入。

同样的判断也适用于你在版本之间注意到的回归。如果一次先前很快的分析突然变慢,那值得报告。

使用 `MAGO_LOG=trace` 重新运行以获取完整的流水线 trace:

```bash
MAGO_LOG=trace mago analyze
```

启用 trace 后,Mago 会:

- 启动一个挂起监视器,标记任何分析时间超过几秒的单个文件。这有助于找出让分析器陷入长时间或死循环的文件。
- 打印并行分析阶段中观察到的最慢的若干文件,从而看到哪些输入主导了总耗时。
- 输出源发现、编译、代码库合并、元数据填充、并行分析、归约等各阶段的耗时,从而判断哪个阶段拖慢了整体。

报告慢速运行或回归时,请附上完整的 trace 输出以及挂起监视器指出的文件。匿名化名称、清除敏感字面量即可,我们只需要复现该输入的形状。
