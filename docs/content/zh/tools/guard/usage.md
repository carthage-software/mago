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
