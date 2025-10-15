---
title: Guard Usage
outline: deep
---

# Using the Guard

The `mago guard` command is your entry point for running all architectural validation checks.

## Running the Guard

To check your entire project against the rules defined in your `mago.toml` file, simply run:

```sh
mago guard
```

Mago will scan your codebase and report any violations it finds.

### Analyzing Specific Paths

You can also run the guard on specific files or directories by passing them as arguments. This is useful for focusing on a particular module or for pre-commit hooks.

```sh
# Analyze only the src/Domain directory
mago guard src/Domain

# Analyze a specific file
mago guard src/UI/Controller/UserController.php
```

## Understanding the Output

The guard produces two types of error reports: **Boundary Breaches** (from the Perimeter Guard) and **Structural Flaws** (from the Structural Guard).

### Example: Boundary Breach

If a part of your code violates a dependency rule, you'll get a "Boundary Breach" error.

Consider this rule in `mago.toml`:
```toml
[[guard.perimeter.rules]]
namespace = "App\\Domain\\"
permit = ["@self", "@native"]
```
This rule states that the `App\Domain` namespace can only depend on itself and native PHP symbols.

Now, if you have the following code:
```php
// src/Domain/Model/User.php
namespace App\Domain\Model;

use App\Infrastructure\Doctrine\Orm\Entity; // <-- Violation!

class User extends Entity {}
```

Running `mago guard` would produce an error like this:

```
error[disallowed-use]: Illegal dependency on `App\Infrastructure\Doctrine\Orm\Entity`
  ┌─ src/Domain/Model/User.php:4:5
  │
4 │ use App\Infrastructure\Doctrine\Orm\Entity;
  │     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ This `use` statement is not allowed by the architectural rules
  │
  = Breach occurred in namespace `App\Domain\Model`.
  = Dependency forbidden by architectural rules
  = The following rule(s) were evaluated but none permitted this dependency: `App\Domain\\`.
  = Help: Update your guard configuration to allow this dependency or refactor the code to remove it.
```

The error clearly shows the forbidden dependency, where it occurred, and which rule was violated.

### Example: Structural Flaw

If a symbol doesn't adhere to a structural rule, you'll get a "Structural Flaw" error.

Consider this rule:
```toml
[[guard.structural.rules]]
on            = "App\\UI\\**\\Controller\\**"
target        = "class"
must-be-final = true
reason        = "Controllers should be final to prevent extension."
```

And this code:
```php
// src/UI/Controller/UserController.php
namespace App\UI\Controller;

class UserController // <-- Violation! Not final.
{
    // ...
}
```

Running `mago guard` would report:

```
error[must-be-final]: Structural flaw in `App\UI\Controller\UserController`
   ┌─ src/UI/Controller/UserController.php:3:7
   │
 3 │ class UserController
   │       ^^^^^^^^^^^^^^ This must be declared as `final`
   │
   = Controllers should be final to prevent extension.
   = Help: Declare this class as `final`.
```The report identifies the symbol, the location, the exact flaw, and the reason provided in the configuration.
