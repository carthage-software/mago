---
title: Guard Configuration Reference
outline: deep
---

# Configuration Reference

`mago guard` is configured under the `[guard]` table in your `mago.toml` file. The configuration is split into two main parts: `[guard.perimeter]` for dependency rules and `[[guard.structural.rules]]` for code convention rules.

## Top-Level `[guard]` Table

This is the main table for the guard tool.

| Option             | Type       | Default     | Description                                                |
| :----------------- | :--------- | :---------- | :--------------------------------------------------------- |
| `mode`             | `string`   | `"default"` | Which guard checks to run. Options: `"default"` (both), `"structural"`, or `"perimeter"`. |
| `excludes`         | `string[]` | `[]`        | A list of paths or glob patterns to exclude from analysis. |
| `baseline`         | `string`   | `null`      | Path to a baseline file to ignore listed issues. When specified, the guard will use this file as the default baseline. |
| `baseline-variant` | `string`   | `"loose"`   | The baseline format variant to use when generating new baselines. Options: `"loose"` (count-based, resilient to line changes) or `"strict"` (exact line matching). See [Baseline Variants](/fundamentals/baseline#baseline-variants) for details. |

### `mode`

Controls which types of guard checks are executed:

- `"default"`: Runs both perimeter and structural guards (this is the default behavior).
- `"structural"`: Runs only structural guard checks (naming conventions, modifiers, inheritance constraints).
- `"perimeter"`: Runs only perimeter guard checks (dependency boundaries, layer restrictions).

```toml
[guard]
mode = "structural"  # Only run structural checks
```

:::tip
You can also override the mode at runtime using the `--structural` or `--perimeter` command-line flags. See the [Command Reference](/tools/guard/command-reference) for details.
:::

:::tip Tool-Specific Excludes
The `excludes` option here is **additive** to the global `source.excludes` defined in the `[source]` section of your configuration. Files excluded globally will always be excluded from guard analysis, and this option allows you to exclude additional files from the guard specifically.

For example:
```toml
[source]
excludes = ["cache/**"]  # Excluded from ALL tools

[guard]
excludes = ["src/ThirdParty/**"]  # Additionally excluded from guard only
```
:::

## Perimeter Guard: `[guard.perimeter]`

This section defines the rules for dependency validation between different parts of your application.

```toml
[guard.perimeter]
# Defines the architectural layers from core to infrastructure.
layering = [
    "CarthageSoftware\\Domain",
    "CarthageSoftware\\Application",
    "CarthageSoftware\\UI",
    "CarthageSoftware\\Infrastructure"
]

# Creates reusable aliases for groups of namespaces.
[guard.perimeter.layers]
core = ["@native", "Psl\\**"]
psr = ["Psr\\**"]
framework = ["Symfony\\**", "Doctrine\\**"]

# Defines dependency rules for specific namespaces.
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

An array of namespaces defining the architectural layers of your application, from the most independent (core) to the outermost layers. This enforces a top-down dependency flow, meaning a layer can only depend on layers defined *before* it in this list. If a dependency points to a layer defined *after* its own, a violation is reported.

### `[guard.perimeter.layers]`

This table allows you to create reusable aliases for groups of namespaces or other paths. These aliases can then be referenced in your permission rules using the `@layer:<name>` syntax.

### `[[guard.perimeter.rules]]`

This is an array of tables where each table defines a dependency rule for a specific part of your codebase.

*   `namespace`: The namespace this rule applies to. This can be a full namespace ending in `\` or the special keyword `@global` for the global namespace.
*   `permit`: A list of permitted dependencies for the `namespace`. This can be a simple string or a detailed object.

#### `permit` Values

The `permit` array accepts a list of "paths". A path can be a special keyword, a namespace, a symbol, or a glob pattern.

| Path Syntax        | Description                                                                                                                                                           |
| :----------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `@global`          | Allows dependencies on symbols defined in the global namespace.                                                                                                       |
| `@all`             | Allows dependencies on any symbol in the entire codebase, including vendor packages. Useful for tests.                                                                |
| `@self` / `@this`  | Allows dependencies on any symbol within the same root namespace as the rule's `namespace`.                                                                             |
| `@native` / `@php` | Allows dependencies on PHP's native (built-in) functions, classes, and constants.                                                                                       |
| `@layer:<name>`    | Allows dependencies on all namespaces and paths defined in the specified layer alias from `[guard.perimeter.layers]`.                                                   |
| `App\Shared\\**`   | A glob pattern. `*` matches any part of a namespace segment, and `**` matches zero or more segments. Allows dependencies on any symbol matching the pattern.             |
| `App\Service`      | An exact, fully-qualified symbol name. Allows a dependency on this specific class, interface, trait, function, or constant.                                            |
| `App\Service\\`    | An exact namespace. Allows dependencies on any symbol directly within this namespace.                                                                                   |

You can also specify permissions in more detail using an object:

```toml
[[guard.perimeter.rules]]
namespace = "DoctrineMigrations\\"
# Allow depending on any class-like symbol, but not functions or constants.
permit = [ { path = "@all", kinds = ["class-like"] } ]
```

*   `path`: Any of the valid path strings described above.
*   `kinds`: An array specifying which *kinds* of symbols are permitted from that path.
    *   `class-like`: Includes classes, interfaces, traits, and enums.
    *   `function`
    *   `constant`
    *   `attribute`

## Structural Guard: `[[guard.structural.rules]]`

This section is for enforcing coding conventions and structural rules. Each table in this array defines a rule that applies to symbols matching a set of selectors.

```toml
[[guard.structural.rules]]
on               = "CarthageSoftware\\UI\\**\\Controller\\**"
target           = "class"
must-be-named    = "*Controller"
must-be-final    = true
must-be-readonly = true
reason           = "Controllers must be final and follow naming conventions."

[[guard.structural.rules]]
on            = "CarthageSoftware\\Domain\\**\\Repository\\**"
target        = "interface"
must-be-named = "*RepositoryInterface"
reason        = "Domain repository interfaces must follow a standard naming convention."

[[guard.structural.rules]]
on            = "CarthageSoftware\\Infrastructure\\**\\Repository\\**"
target        = "class"
must-be-final = true
must-extend   = "CarthageSoftware\\Infrastructure\\Shared\\Repository\\AbstractRepository"
reason        = "Infrastructure repositories must extend our abstract class."

[[guard.structural.rules]]
on          = "CarthageSoftware\\Domain\\**\\Enum\\**"
must-be     = ["enum"]
reason      = "This namespace is designated for enums only."
```

### Selector Keys

These keys determine which symbols a rule applies to.

| Key      | Description                                                                                                                            |
| :------- | :------------------------------------------------------------------------------------------------------------------------------------- |
| `on`     | **Required.** A glob pattern matching the fully-qualified name of the symbols to target.                                                |
| `not-on` | An optional glob pattern to exclude symbols that would otherwise be matched by `on`.                                                     |
| `target` | An optional filter to apply the rule only to a specific kind of symbol. <br/>**Values:** `class`, `interface`, `trait`, `enum`, `function`, `constant`. |

### Constraint Keys

These keys define the architectural constraints to enforce on the selected symbols.

| Key                 | Description                                                                                                                            |
| :------------------ | :------------------------------------------------------------------------------------------------------------------------------------- |
| `must-be`             | Restricts the `on` namespace to contain *only* the specified symbol kinds. <br/>**Values:** `class`, `interface`, `trait`, `enum`, `function`, `constant`. |
| `must-be-named`       | Enforces a naming convention using a glob pattern (e.g., `*Controller`).                                                               |
| `must-be-final`       | A boolean. If `true`, the symbol must be `final`. If `false`, it must *not* be `final`.                                                 |
| `must-be-abstract`    | A boolean. If `true`, the symbol must be `abstract`. If `false`, it must *not* be `abstract`.                                           |
| `must-be-readonly`    | A boolean. If `true`, the symbol must be `readonly`. If `false`, it must *not* be `readonly`.                                           |
| `must-implement`      | Enforces that a class must implement one or more interfaces.                                                                           |
| `must-extend`         | Enforces that a class must extend a specific parent class.                                                                             |
| `must-use-trait`      | Enforces that a class or trait must use one or more traits.                                                                            |
| `must-use-attribute`  | Enforces that a symbol must have one or more attributes.                                                                               |
| `reason`              | An optional human-readable string explaining why the rule exists. This will be displayed in the error message.                         |

#### Inheritance Constraints (`must-implement`, `must-extend`, etc.)

These constraints can be a single string, an array of strings (for an `AND` condition), or an array of arrays of strings (for an `OR` of `AND`s).

```toml
# Must extend a single class
must-extend = "App\\BaseClass"

# Must implement ALL of these interfaces
must-implement = ["App\\InterfaceA", "App\\InterfaceB"]

# Must extend (AbstractA AND AbstractB) OR (AbstractC)
must-extend = [
    ["App\\AbstractA", "App\\AbstractB"],
    ["App\\AbstractC"],
]

# Must not implement any interface
must-implement = "@nothing"
```
