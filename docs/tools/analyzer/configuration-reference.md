---
title: Analyzer configuration Reference
---

# Configuration reference

**Mago**'s analyzer is highly configurable, allowing you to tailor the analysis to your project's specific needs. All settings go under the `[analyzer]` table in your `mago.toml` file.

```toml
[analyzer]
# Disable a specific issue category
redundancy-issues = false

# Ignore a specific error code across the whole project
ignore = ["mixed-argument"]

# Use a baseline file to ignore existing issues
baseline = "analyzer-baseline.toml"
```

## General options

| Option             | Type       | Default   | Description                                                |
| :----------------- | :--------- | :-------- | :--------------------------------------------------------- |
| `excludes`         | `string[]` | `[]`      | A list of paths or glob patterns to exclude from analysis. |
| `ignore`           | `string[]` | `[]`      | A list of specific issue codes to ignore globally.         |
| `baseline`         | `string`   | `null`    | Path to a baseline file to ignore listed issues. When specified, the analyzer will use this file as the default baseline, eliminating the need to pass `--baseline` on every run. Command-line `--baseline` arguments will override this setting. |
| `baseline-variant` | `string`   | `"loose"` | The baseline format variant to use when generating new baselines. Options: `"loose"` (count-based, resilient to line changes) or `"strict"` (exact line matching). See [Baseline Variants](/fundamentals/baseline#baseline-variants) for details. |

:::tip Tool-Specific Excludes
The `excludes` option here is **additive** to the global `source.excludes` defined in the `[source]` section of your configuration. Files excluded globally will always be excluded from analysis, and this option allows you to exclude additional files from the analyzer specifically.

For example:
```toml
[source]
excludes = ["cache/**"]  # Excluded from ALL tools

[analyzer]
excludes = ["tests/**/*.php"]  # Additionally excluded from analyzer only
```
:::

## Feature flags

These flags control specific, powerful analysis capabilities.

| Option                                | Default | Description                                                                                          |
| :------------------------------------ | :------ | :--------------------------------------------------------------------------------------------------- |
| `find-unused-expressions`             | `true`  | Find and report expressions whose results are not used (e.g., `$a + $b;`).                           |
| `find-unused-definitions`             | `true`  | Find and report unused definitions (e.g., private methods that are never called).                    |
| `analyze-dead-code`                   | `false` | Analyze code that appears to be unreachable.                                                         |
| `memoize-properties`                  | `true`  | Track the literal values of class properties. Improves type inference but may increase memory usage. |
| `allow-possibly-undefined-array-keys` | `true`  | Allow accessing array keys that may not be defined without reporting an issue.                       |
| `check-throws`                        | `false` | Check for unhandled thrown exceptions that are not caught or documented with `@throws`.              |
| `check-missing-override`              | `false` | Check for missing `#[Override]` attributes on overriding methods (PHP 8.3+).                         |
| `find-unused-parameters`              | `false` | Find and report unused function/method parameters.                                                   |
| `strict-list-index-checks`            | `false` | When `true`, requires any integer used as a `list` index to be provably non-negative.                |
| `no-boolean-literal-comparison`       | `false` | When `true`, disallows direct comparison to boolean literals (e.g., `$a === true`).                  |
| `check-missing-type-hints`            | `false` | When `true`, reports missing type hints on parameters, properties, and return types.                 |
| `check-closure-missing-type-hints`    | `false` | When `true`, checks closures for missing type hints when `check-missing-type-hints` is enabled.      |
| `check-arrow-function-missing-type-hints` | `false` | When `true`, checks arrow functions for missing type hints when `check-missing-type-hints` is enabled. |
| `register-super-globals`              | `true`  | Automatically register PHP superglobals (e.g., `$_GET`, `$_POST`) for analysis.                      |
| `trust-existence-checks`              | `true`  | When `true`, narrows types based on `method_exists()`, `property_exists()`, `function_exists()`, and `defined()` checks. |
| `check-property-initialization`       | `false` | When `true`, checks that typed properties are initialized in constructors or class initializers.      |
| `check-use-statements`                | `false` | When `true`, reports use statements that import non-existent classes, functions, or constants.        |

## Property initialization

These options control how the analyzer checks property initialization.

| Option                           | Type       | Default | Description                                                                         |
| :------------------------------- | :--------- | :------ | :---------------------------------------------------------------------------------- |
| `check-property-initialization`  | `bool`     | `false` | Enable/disable property initialization checking entirely.                            |
| `class-initializers`             | `string[]` | `[]`    | Method names treated as class initializers (like `__construct`).                     |

### How it works

When `check-property-initialization` is enabled, the analyzer reports:
- **`missing-constructor`**: Classes with typed properties that have no constructor to initialize them
- **`uninitialized-property`**: Typed properties not initialized in the constructor

The `class-initializers` setting allows you to specify additional methods that should be treated as initializers. Properties initialized in these methods count as "definitely initialized", just like in `__construct`. This is useful for frameworks that use lifecycle methods like:
- PHPUnit's `setUp()` method for test classes
- Framework-specific `boot()` or `initialize()` methods

### Example

```toml
[analyzer]
# Treat setUp as a class initializer (for PHPUnit tests)
class-initializers = ["setUp", "initialize", "boot"]
```

With this configuration, the following code won't trigger false positives:

```php
class MyTest extends TestCase
{
    private string $name;

    protected function setUp(): void
    {
        // Property initialized in setUp - no error reported
        $this->name = "test";
    }
}
```

### Enabling property initialization checks

Property initialization checking is disabled by default. To enable it:

```toml
[analyzer]
check-property-initialization = true
```

This enables both `missing-constructor` and `uninitialized-property` issues.

## Exception filtering

When `check-throws` is enabled, you can fine-tune which exceptions should be ignored using these options:

| Option                       | Type       | Default | Description                                                                                      |
| :--------------------------- | :--------- | :------ | :----------------------------------------------------------------------------------------------- |
| `unchecked-exceptions`       | `string[]` | `[]`    | Exceptions to ignore **including all their subclasses** (hierarchy-aware).                       |
| `unchecked-exception-classes`| `string[]` | `[]`    | Exceptions to ignore **only as exact class matches** (not subclasses or parent classes).         |

### How it works

- **`unchecked-exceptions`**: When you add an exception class here, that exception and all classes that extend it will be ignored. This is useful for ignoring entire exception hierarchies, like all logic exceptions.

- **`unchecked-exception-classes`**: When you add an exception class here, only that exact class is ignored. Subclasses and parent classes are still checked. This is useful when you want to ignore a specific exception without affecting related exceptions.

### Example

```toml
[analyzer]
check-throws = true

# Ignore LogicException and ALL its subclasses:
# - InvalidArgumentException
# - OutOfRangeException
# - DomainException
# - etc.
unchecked-exceptions = [
    "LogicException",
    "Psl\\Type\\Exception\\ExceptionInterface",
]

# Ignore ONLY this specific exception class (not its parent or child classes)
unchecked-exception-classes = [
    "Psl\\File\\Exception\\FileNotFoundException",
]
```

In this example:
- Any unhandled `LogicException` or its subclasses (like `InvalidArgumentException`) won't be reported
- Any class implementing `Psl\Type\Exception\ExceptionInterface` won't be reported
- Only the exact `Psl\File\Exception\FileNotFoundException` is ignored, but other file exceptions would still be reported

:::tip When to use which option
Use `unchecked-exceptions` when you want to ignore an entire category of exceptions, such as logic errors that indicate programming mistakes rather than runtime issues.

Use `unchecked-exception-classes` when you want to ignore a specific exception but still want to track its parent or sibling exceptions.
:::

## Plugins

Plugins extend the analyzer with specialized type information for libraries and frameworks. They provide accurate type inference for functions that would otherwise return generic types.

| Option                   | Type       | Default | Description                                                              |
| :----------------------- | :--------- | :------ | :----------------------------------------------------------------------- |
| `disable-default-plugins`| `bool`     | `false` | Disable all default plugins. Only explicitly listed plugins will be used.|
| `plugins`                | `string[]` | `[]`    | List of plugins to enable (by name or alias).                            |

### Available plugins

| Plugin ID   | Aliases                                    | Default | Description                                                    |
| :---------- | :----------------------------------------- | :------ | :------------------------------------------------------------- |
| `stdlib`    | `standard`, `std`, `php-stdlib`            | Enabled | Type providers for PHP built-in functions (`strlen`, `array_*`, `json_*`, etc.) |
| `psl`       | `php-standard-library`, `azjezz-psl`       | Disabled| Type providers for [azjezz/psl](https://github.com/azjezz/psl) package |
| `flow-php`  | `flow`, `flow-etl`                         | Disabled| Type providers for [flow-php/etl](https://github.com/flow-php/etl) package |

### How plugins work

Plugins provide "type providers" that give the analyzer precise type information for library functions. For example, the `stdlib` plugin knows that:

- `array_filter($array)` returns the same array type but potentially with fewer elements
- `json_decode($json, true)` returns `array<string, mixed>` when the second argument is `true`
- `strlen($string)` returns `int<0, max>`

Without plugins, these functions would return less precise types like `mixed` or `array`.

### Example configurations

#### Using default plugins only

By default, the `stdlib` plugin is enabled:

```toml
[analyzer]
# No configuration needed - stdlib is enabled by default
```

#### Enabling additional plugins

```toml
[analyzer]
plugins = ["psl", "flow-php"]
```

#### Disabling all plugins

```toml
[analyzer]
disable-default-plugins = true
```

#### Using only specific plugins

```toml
[analyzer]
disable-default-plugins = true
plugins = ["psl"]  # Only enable PSL, not stdlib
```

:::tip Plugin aliases
You can use any of the plugin aliases for convenience. For example, `plugins = ["std"]` is equivalent to `plugins = ["stdlib"]`.
:::

## Strict mode

The analyzer can be configured to be more or less strict depending on your project's needs. This section describes how to configure the analyzer for maximum strictness.

### Maximum strictness configuration

For the strictest possible analysis, use the following configuration:

```toml
[analyzer]
# Enable all checks
find-unused-expressions = true
find-unused-definitions = true
analyze-dead-code = true
check-throws = true
check-missing-override = true
find-unused-parameters = true
check-missing-type-hints = true
check-closure-missing-type-hints = true
check-arrow-function-missing-type-hints = true

# Enable strict checks
strict-list-index-checks = true
no-boolean-literal-comparison = true

# Disable lenient behaviors
allow-possibly-undefined-array-keys = false
trust-existence-checks = false
```

### Strictness options explained

#### Type hint enforcement

| Option | Strict Value | Effect |
| :----- | :----------- | :----- |
| `check-missing-type-hints` | `true` | Reports missing type hints on function parameters, return types, and class properties. |
| `check-closure-missing-type-hints` | `true` | Also checks closures for missing type hints (requires `check-missing-type-hints`). |
| `check-arrow-function-missing-type-hints` | `true` | Also checks arrow functions for missing type hints (requires `check-missing-type-hints`). |

#### Array access strictness

| Option | Strict Value | Effect |
| :----- | :----------- | :----- |
| `allow-possibly-undefined-array-keys` | `false` | Reports errors when accessing array keys that may not exist. |
| `strict-list-index-checks` | `true` | Requires list indices to be provably non-negative (`int<0, max>`). |

#### Runtime check behavior

| Option | Strict Value | Effect |
| :----- | :----------- | :----- |
| `trust-existence-checks` | `false` | Ignores `method_exists()`, `property_exists()` checks; requires explicit type hints. |

When `trust-existence-checks` is enabled (the default), the analyzer narrows types based on runtime existence checks:

```php
function process(object $obj): mixed
{
    // With trust-existence-checks = true (default):
    // No warning - method existence is verified at runtime
    if (method_exists($obj, 'toArray')) {
        return $obj->toArray();
    }

    // With trust-existence-checks = false:
    // Warning reported - explicit type hints required
    return null;
}
```

#### Code quality checks

| Option | Strict Value | Effect |
| :----- | :----------- | :----- |
| `find-unused-expressions` | `true` | Reports expressions whose results are discarded (e.g., `$a + $b;`). |
| `find-unused-definitions` | `true` | Reports unused private methods, variables, and other definitions. |
| `analyze-dead-code` | `true` | Analyzes and reports on unreachable code paths. |
| `check-missing-override` | `true` | Reports missing `#[Override]` attributes on overriding methods (PHP 8.3+). |
| `find-unused-parameters` | `true` | Reports unused function/method parameters. |
| `no-boolean-literal-comparison` | `true` | Disallows comparisons like `$a === true` or `$b == false`. |

#### Exception handling

| Option | Strict Value | Effect |
| :----- | :----------- | :----- |
| `check-throws` | `true` | Reports unhandled exceptions not caught or documented with `@throws`. |

### Lenient mode

For a more lenient analysis (useful for legacy codebases or gradual adoption), use:

```toml
[analyzer]
# Disable strict checks
check-missing-type-hints = false
strict-list-index-checks = false
no-boolean-literal-comparison = false

# Enable lenient behaviors
allow-possibly-undefined-array-keys = true
trust-existence-checks = true

# Optionally disable some checks
check-throws = false
```

:::tip Gradual adoption
When introducing Mago to an existing codebase, start with lenient settings and a [baseline](/fundamentals/baseline). Gradually enable stricter options as you improve the codebase.
:::

## Performance tuning

The analyzer uses internal thresholds to balance analysis depth against performance. These thresholds control how deeply the type inference engine explores complex logical formulas. All settings go under `[analyzer.performance]` in your `mago.toml` file.

| Option                                | Type  | Default | Description                                                              |
| :------------------------------------ | :---- | :------ | :----------------------------------------------------------------------- |
| `saturation-complexity-threshold`     | `u16` | `8192`  | Maximum clauses during CNF saturation.                                   |
| `disjunction-complexity-threshold`    | `u16` | `4096`  | Maximum clauses per side in OR operations.                               |
| `negation-complexity-threshold`       | `u16` | `4096`  | Maximum cumulative complexity when negating formulas.                    |
| `consensus-limit-threshold`           | `u16` | `256`   | Upper limit for consensus optimization passes.                           |
| `formula-size-threshold`              | `u16` | `512`   | Maximum logical formula size before simplification is skipped.           |
| `string-concat-combination-threshold` | `u16` | `512`  | Maximum combinations to track during string concatenation.               |

### When to adjust thresholds

Most projects work well with the default values. Consider adjusting these thresholds if:

- **Analysis is too slow**: Reduce thresholds to improve speed at the cost of some type inference precision
- **Type inference is imprecise**: Increase thresholds for deeper analysis on complex conditional code

### Understanding the thresholds

The analyzer converts type constraints into CNF (Conjunctive Normal Form) logical formulas. These formulas can grow exponentially with complex conditional logic. The thresholds prevent runaway computation.

- **Saturation complexity**: Controls how many clauses the formula simplifier will process during saturation. When exceeded, simplification stops early.
- **Disjunction complexity**: Limits clause explosion when combining OR conditions. Complex union types or many conditional branches may hit this limit.
- **Negation complexity**: Limits expansion when negating formulas (e.g., for `else` branches). Deeply nested conditions may hit this limit.
- **Consensus limit**: Controls an optimization pass that detects logical tautologies. Higher values may find more simplifications.
- **Formula size**: Overall limit on formula complexity before the analyzer falls back to simpler inference.
- **String concat combinations**: Limits the number of possible string literal combinations during concatenation to prevent exponential blowup in large concatenation chains.

### Example configurations

#### Fast analysis (reduced precision)

For large codebases where speed is more important than deep type inference:

```toml
[analyzer.performance]
saturation-complexity-threshold = 2048
disjunction-complexity-threshold = 1024
negation-complexity-threshold = 1024
consensus-limit-threshold = 64
formula-size-threshold = 128
string-concat-combination-threshold = 512
```

#### Deep analysis (slower, more precise)

For smaller codebases or CI pipelines where thorough analysis is important:

```toml
[analyzer.performance]
saturation-complexity-threshold = 16384
disjunction-complexity-threshold = 8192
negation-complexity-threshold = 8192
consensus-limit-threshold = 512
formula-size-threshold = 1024
string-concat-combination-threshold = 8192
```

:::warning Performance impact
Increasing these thresholds can significantly impact analysis time on codebases with complex conditional logic. Test on your codebase before deploying to CI.
:::
