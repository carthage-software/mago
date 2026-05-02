+++
title = "Guard usage"
description = "Running mago guard and reading what it reports."
nav_order = 20
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Using the guard

To check the project against the rules in `mago.toml`:

```sh
mago guard
```

To check a single directory or file:

```sh
mago guard src/Domain
mago guard src/UI/Controller/UserController.php
```

Paths passed on the command line replace the `paths` from `mago.toml` for that run.

## Reading the output

The guard reports two kinds of issues: boundary breaches from the perimeter guard and structural flaws from the structural guard.

### Boundary breach

Given this rule:

```toml
[[guard.perimeter.rules]]
namespace = "App\\Domain\\"
permit = ["@self", "@native"]
```

And this code:

```php
namespace App\Domain\Model;

use App\Infrastructure\Doctrine\Orm\Entity;

class User extends Entity {}
```

The guard reports:

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

### Structural flaw

Given this rule:

```toml
[[guard.structural.rules]]
on = "App\\UI\\**\\Controller\\**"
target = "class"
must-be-final = true
reason = "Controllers should be final to prevent extension."
```

And this code:

```php
namespace App\UI\Controller;

class UserController
{
}
```

The guard reports:

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

Each report identifies the symbol, the location, the exact violation, and the `reason` from the configuration when one was provided.
