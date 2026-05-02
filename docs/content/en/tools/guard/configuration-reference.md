+++
title = "Guard configuration reference"
description = "Every option Mago accepts under [guard]."
nav_order = 40
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Configuration reference

Settings live under `[guard]` in `mago.toml`. The configuration has two parts: `[guard.perimeter]` for dependency rules and `[[guard.structural.rules]]` for structural conventions.

## Top-level options

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `mode` | `string` | `"default"` | Which checks to run. One of `"default"`, `"structural"`, `"perimeter"`. |
| `excludes` | `string[]` | `[]` | Paths or glob patterns to exclude from analysis. Additive to `[source].excludes`. |
| `baseline` | `string` | unset | Path to a baseline file. Equivalent to passing `--baseline` on every run. |
| `baseline-variant` | `string` | `"loose"` | Format for newly generated baselines. `"loose"` (count-based) or `"strict"` (exact line matching). See [baseline](/fundamentals/baseline/). |
| `minimum-fail-level` | `string` | `"error"` | Minimum severity that causes a non-zero exit. One of `"note"`, `"help"`, `"warning"`, `"error"`. Overridden by `--minimum-fail-level`. |

`mode` controls which half of the guard runs:

- `"default"` runs both halves.
- `"structural"` runs only structural checks.
- `"perimeter"` runs only perimeter checks.

```toml
[guard]
mode = "structural"
```

The `--structural` and `--perimeter` flags override the configured mode. See the [command reference](/tools/guard/command-reference/).

`excludes` here is added to whatever you set in `[source].excludes`; it never narrows the global list.

```toml
[source]
excludes = ["cache/**"]

[guard]
excludes = ["src/ThirdParty/**"]
```

## Perimeter guard

The perimeter section defines dependency rules between parts of the project.

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

An ordered list of namespaces, from the most independent core down to the outermost layer. Each layer can only depend on layers defined before it. A dependency that points to a layer further out triggers a violation.

### Layer aliases

`[guard.perimeter.layers]` defines reusable groups of namespaces and paths, referenced from rules with `@layer:<name>`.

### Rules

Each `[[guard.perimeter.rules]]` table defines one rule:

- `namespace`: the namespace this rule applies to. Either a namespace ending in `\` or the special keyword `@global` for the global namespace.
- `permit`: the dependencies that are allowed. Either a list of strings or a list of detailed objects.

#### `permit` values

`permit` accepts paths. A path can be a keyword, a namespace, a symbol, or a glob pattern.

| Path | Description |
| :--- | :--- |
| `@global` | Symbols defined in the global namespace. |
| `@all` | Any symbol anywhere in the project, including vendor packages. Useful for tests. |
| `@self` / `@this` | Any symbol within the same root namespace as the rule's `namespace`. |
| `@native` / `@php` | PHP's built-in functions, classes, and constants. |
| `@layer:<name>` | All namespaces and paths in the named alias from `[guard.perimeter.layers]`. |
| `App\Shared\\**` | Glob pattern. `*` matches a single namespace segment, `**` matches zero or more. |
| `App\Service` | Exact fully qualified symbol name. |
| `App\Service\\` | Exact namespace. Allows symbols directly within it. |

You can narrow a permission by symbol kind using an object form:

```toml
[[guard.perimeter.rules]]
namespace = "DoctrineMigrations\\"
permit = [{ path = "@all", kinds = ["class-like"] }]
```

- `path`: any of the path forms above.
- `kinds`: which kinds of symbols are permitted. Values are `class-like` (covers classes, interfaces, traits, enums), `function`, `constant`, and `attribute`.

## Structural guard

`[[guard.structural.rules]]` defines structural conventions. Each entry combines selectors that pick which symbols to inspect with constraints that the selected symbols must satisfy.

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

### Selectors

| Key | Description |
| :--- | :--- |
| `on` | Required. Glob pattern matching the fully qualified name of the symbols this rule applies to. |
| `not-on` | Optional glob pattern excluding symbols that would otherwise match `on`. |
| `target` | Optional filter restricting the rule to one symbol kind. One of `class`, `interface`, `trait`, `enum`, `function`, `constant`. |

### Constraints

| Key | Description |
| :--- | :--- |
| `must-be` | Restrict the selected namespace to contain only the listed symbol kinds. Values: `class`, `interface`, `trait`, `enum`, `function`, `constant`. |
| `must-be-named` | Glob pattern the symbol name must match (e.g. `*Controller`). |
| `must-be-final` | Boolean. `true` requires `final`; `false` forbids it. |
| `must-be-abstract` | Boolean. `true` requires `abstract`; `false` forbids it. |
| `must-be-readonly` | Boolean. `true` requires `readonly`; `false` forbids it. |
| `must-implement` | One or more interfaces the class must implement. |
| `must-extend` | A class the symbol must extend. |
| `must-use-trait` | One or more traits the symbol must use. |
| `must-use-attribute` | One or more attributes the symbol must carry. |
| `reason` | Human-readable explanation shown in error messages. |

#### Inheritance constraint shapes

`must-implement`, `must-extend`, `must-use-trait`, and `must-use-attribute` accept a single string, an array of strings (AND), or an array of arrays of strings (OR of ANDs). The literal `"@nothing"` forbids any value.

```toml
must-extend = "App\\BaseClass"

must-implement = ["App\\InterfaceA", "App\\InterfaceB"]

must-extend = [
    ["App\\AbstractA", "App\\AbstractB"],
    ["App\\AbstractC"],
]

must-implement = "@nothing"
```
