+++
title = "Guard 使用"
description = "运行 mago guard 并理解它报告的内容。"
nav_order = 20
nav_section = "工具"
nav_subsection = "Guard"
+++
# 使用 guard

按 `mago.toml` 中的规则检查项目:

```sh
mago guard
```

检查单个目录或单个文件:

```sh
mago guard src/Domain
mago guard src/UI/Controller/UserController.php
```

在命令行传入的路径,会替代该次运行所用的 `mago.toml` 中的 `paths`。

## 解读输出

guard 报告两类问题:来自边界 guard 的边界违规,以及来自结构 guard 的结构缺陷。

### 边界违规

给定如下规则:

```toml
[[guard.perimeter.rules]]
namespace = "App\\Domain\\"
permit = ["@self", "@native"]
```

以及如下代码:

```php
namespace App\Domain\Model;

use App\Infrastructure\Doctrine\Orm\Entity;

class User extends Entity {}
```

guard 会报告:

```
error[disallowed-use]: Illegal dependency on `App\Infrastructure\Doctrine\Orm\Entity`
 ┌─ src/Domain/Model/User.php:4:5
 │
4 │ use App\Infrastructure\Doctrine\Orm\Entity;
 │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ This `use` statement is not allowed by the architectural rules
 │
 = Breach occurred in namespace `App\Domain\Model`.
 = Dependency forbidden by architectural rules
 = The following rule(s) were evaluated but none permitted this dependency: `App\Domain\\`.
 = Help: Update your guard configuration to allow this dependency or refactor the code to remove it.
```

### 限制依赖可以从哪里使用

边界限制用于保护目标依赖,限定哪些来源命名空间可以使用它。下面的规则只允许 `App\Http\Controllers\` 下的代码使用基础控制器:

```toml
[[guard.perimeter.restrictions]]
dependency = "App\\Http\\Controllers\\Controller"
allow-from = ["App\\Http\\Controllers\\"]
```

下面的规则会在整个 `App\` 中禁止某个外部 trait,即使普通边界规则允许该依赖:

```toml
[[guard.perimeter.restrictions]]
dependency = "Illuminate\\Foundation\\Bus\\Dispatchable"
deny-from = ["App\\"]
```

限制适合表达有针对性的禁用规则。未配置普通边界规则或 layering 时,不匹配任何限制的依赖仍然允许使用。

### 结构缺陷

给定如下规则:

```toml
[[guard.structural.rules]]
on = "App\\UI\\**\\Controller\\**"
target = "class"
must-be-final = true
reason = "Controllers should be final to prevent extension."
```

以及如下代码:

```php
namespace App\UI\Controller;

class UserController
{
}
```

guard 会报告:

```
error[must-be-final]: Structural flaw in `App\UI\Controller\UserController`
 ┌─ src/UI/Controller/UserController.php:3:7
 │
 3 │ class UserController
 │ ^^^^^^^^^^^^^^ This must be declared as `final`
 │
 = Controllers should be final to prevent extension.
 = Help: Declare this class as `final`.
```

每条报告都会标识涉及的符号、位置、确切的违规内容,以及配置中提供的 `reason`(若提供)。

### 限制公共方法

使用 `only-public-methods` 可以限制匹配类直接声明的公共 API:

```toml
[[guard.structural.rules]]
on = "App\\Http\\Controllers\\**"
target = "class"
only-public-methods = ["__construct", "__invoke"]
```

任何其他直接声明的公共方法都会产生 `only-public-methods` 结构缺陷。私有和受保护方法仍然允许。配置中的方法名表示允许存在,而不是要求必须存在;继承的方法和 trait 提供的方法不会被检查。
