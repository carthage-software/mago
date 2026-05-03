+++
title = "Analyzer configuration reference"
description = "Every option Mago accepts under [analyzer]."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Analyzer"
+++
# Configuration reference

Settings live under `[analyzer]` in `mago.toml`.

```toml
[analyzer]
ignore = ["mixed-argument"]
baseline = "analyzer-baseline.toml"
```

## General options

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `excludes` | `string[]` | `[]` | Paths or glob patterns to exclude from analysis. Additive to `[source].excludes`. |
| `ignore` | `(string \| object)[]` | `[]` | Issue codes to ignore, optionally scoped to specific paths. See below. |
| `baseline` | `string` | unset | Path to a baseline file. Equivalent to passing `--baseline` on every run. The CLI flag overrides this. |
| `baseline-variant` | `string` | `"loose"` | Format for newly generated baselines. `"loose"` (count-based) or `"strict"` (exact line matching). See [baseline](/fundamentals/baseline/). |
| `minimum-fail-level` | `string` | `"error"` | Minimum severity that causes a non-zero exit. One of `"note"`, `"help"`, `"warning"`, `"error"`. Overridden by `--minimum-fail-level`. |

`excludes` here is added to whatever you set in `[source].excludes`; it never narrows the global list.

```toml
[source]
excludes = ["cache/**"]

[analyzer]
excludes = ["tests/**/*.php"]
```

### Path-scoped ignoring

`ignore` accepts plain strings, single-path objects, and multi-path objects, mixed freely:

```toml
[analyzer]
ignore = [
    "mixed-argument",
    { code = "missing-return-type", in = "tests/" },
    { code = "unused-parameter", in = ["tests/", "src/Generated/"] },
]
```

Each entry in `in` is either a directory or file prefix, or a glob pattern. Any value containing `*`, `?`, `[`, or `{` is treated as a glob and matched against the full relative path; everything else is matched as a prefix. `"tests"` and `"tests/"` both match every file under `tests`.

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

Glob matching honours the project-wide settings under `[source.glob]`, so toggles like `literal-separator` and `case-insensitive` apply here as well.

`excludes` and `ignore` are not the same. `excludes` removes files from analysis entirely, so they are not parsed for type information. `ignore` still analyses the file but suppresses the listed codes in the output.

## Feature flags

These flags toggle individual analyses. Defaults are tuned for everyday use; flip them on as your codebase tightens up.

| Option | Default | Description |
| :--- | :--- | :--- |
| `find-unused-expressions` | `true` | Report expressions whose result is discarded, like `$a + $b;`. |
| `find-unused-definitions` | `true` | Report private definitions that are never referenced. |
| `find-overly-wide-return-types` | `false` | Warn when a declared return type contains a branch the body never produces, like `: string\|false` on a function that always returns a string. Available since 1.20.0. |
| `analyze-dead-code` | `false` | Analyse code that appears unreachable. |
| `memoize-properties` | `true` | Track literal property values for sharper inference, at the cost of some memory. |
| `allow-possibly-undefined-array-keys` | `true` | **Deprecated.** Allow accessing keys that may be missing without flagging it. Setting this to `false` warns on `array<K, V>` reads with a single literal key but does not widen the type to `T\|null`. Use `strict-array-index-existence` instead. |
| `check-throws` | `false` | Report exceptions that are not caught and not declared with `@throws`. |
| `check-missing-override` | `false` | Report missing `#[Override]` attributes on overriding methods (PHP 8.3+). |
| `find-unused-parameters` | `false` | Report parameters that are never read. |
| `strict-list-index-checks` | `false` | Require any integer used as a list index to be provably non-negative. |
| `strict-array-index-existence` | `false` | Treat array/list reads whose key is not provably present as `T\|null` and emit a `possibly-undefined-{int,string}-array-index` warning. Replaces `allow-possibly-undefined-array-keys = false`. |
| `allow-array-truthy-operand` | `false` | Accept arrays as operands of `&&`, `\|\|`, and `xor` without `invalid-operand`. Standalone `if ($array)` is unaffected and never warns. |
| `no-boolean-literal-comparison` | `false` | Disallow direct comparisons to boolean literals like `$a === true`. |
| `check-missing-type-hints` | `false` | Report missing type hints on parameters, properties, and return types. |
| `check-closure-missing-type-hints` | `false` | Extend the type-hint check to closures (requires `check-missing-type-hints`). |
| `check-arrow-function-missing-type-hints` | `false` | Extend the type-hint check to arrow functions (requires `check-missing-type-hints`). |
| `allow-implicit-pipe-callable-types` | `false` | Skip the closure / arrow-function type-hint checks when the callable is the right-hand side of `\|>`. |
| `register-super-globals` | `true` | Register PHP superglobals like `$_GET` and `$_POST` automatically. |
| `trust-existence-checks` | `true` | Narrow types based on `method_exists()`, `property_exists()`, `function_exists()`, and `defined()`. |
| `check-property-initialization` | `false` | Verify that typed properties are initialised in a constructor or class initialiser. |
| `check-use-statements` | `false` | Report use statements that import non-existent classes, functions, or constants. |
| `check-name-casing` | `false` | Report incorrect casing when referencing classes, functions, etc. Helps prevent autoload failures on case-sensitive filesystems. |
| `enforce-class-finality` | `false` | Report classes that are not `final`, `abstract`, or annotated `@api` and have no children. |
| `require-api-or-internal` | `false` | Require abstract classes, interfaces, and traits to be annotated `@api` or `@internal`. |
| `check-experimental` | `false` | Report use of `@experimental` symbols from non-experimental contexts. Available since 1.19.0. |
| `allow-side-effects-in-conditions` | `true` | When `false`, report calls to impure functions inside `if`, `while`, `for`, ternary, or `match` conditions. |

## Property initialization

When `check-property-initialization` is enabled, the analyzer reports two issues:

- `missing-constructor` for classes with typed properties and no constructor.
- `uninitialized-property` for typed properties not assigned in the constructor.

`class-initializers` lets you mark additional methods that should count as initialisers, alongside `__construct`. Properties assigned in those methods are treated as definitely initialised. This is useful for frameworks that use lifecycle methods.

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `class-initializers` | `string[]` | `[]` | Method names treated as class initialisers. |

```toml
[analyzer]
check-property-initialization = true
class-initializers = ["setUp", "initialize", "boot"]
```

With this configuration, the following code does not trigger a false positive:

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

## Exception filtering

When `check-throws` is enabled, two options let you skip specific exceptions.

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `unchecked-exceptions` | `string[]` | `[]` | Exceptions to ignore, including all subclasses (hierarchy-aware). |
| `unchecked-exception-classes` | `string[]` | `[]` | Exceptions to ignore as exact class matches only. Subclasses and parents are still checked. |

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

Use `unchecked-exceptions` to silence an entire category, like every `LogicException` subclass. Use `unchecked-exception-classes` when you want to ignore one specific exception while still tracking siblings and parents.

## Experimental API detection

Set `check-experimental = true` to flag use of `@experimental` symbols from non-experimental code. Mark the symbol with the PHPDoc tag:

```php
/** @experimental */
class UnstableApi {}

/** @experimental */
function beta_feature(): void {}
```

The analyzer warns when these are used from stable code:

```php
new UnstableApi();              // warning
beta_feature();                 // warning
class MyService extends UnstableApi {}  // warning
```

Use from another experimental context is allowed:

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

## Plugins

Plugins ship type providers for libraries and frameworks, so functions return precise types instead of generic ones.

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `disable-default-plugins` | `bool` | `false` | Disable all default plugins. Only the names you list in `plugins` are active. |
| `plugins` | `string[]` | `[]` | Plugins to enable, by name or alias. |

### Available plugins

| Plugin | Aliases | Default | Description |
| :--- | :--- | :--- | :--- |
| `stdlib` | `standard`, `std`, `php-stdlib` | enabled | PHP built-in functions: `strlen`, `array_*`, `json_*`, and friends. |
| `psl` | `php-standard-library`, `azjezz-psl` | disabled | [php-standard-library](https://github.com/php-standard-library/php-standard-library). |
| `flow-php` | `flow`, `flow-etl` | disabled | [flow-php/etl](https://github.com/flow-php/etl). |
| `psr-container` | `psr-11` | disabled | [psr/container](https://github.com/php-fig/container). |

For example, the `stdlib` plugin teaches the analyzer that `strlen($s)` returns `int<0, max>`, that `json_decode($json, true)` returns `array<string, mixed>`, and that `array_filter($array)` keeps the input shape but possibly drops elements.

### Examples

Use the defaults (just `stdlib`):

```toml
[analyzer]
```

Enable additional plugins:

```toml
[analyzer]
plugins = ["psl", "flow-php", "psr-container"]
```

Disable everything:

```toml
[analyzer]
disable-default-plugins = true
```

Use only one plugin:

```toml
[analyzer]
disable-default-plugins = true
plugins = ["psl"]
```

Plugin aliases work everywhere, so `plugins = ["std"]` is the same as `plugins = ["stdlib"]`.

## Strict mode

The analyzer runs at a moderate strictness by default. Crank it up by enabling more checks; ease it off for legacy code.

### Maximum strictness

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
strict-array-index-existence = true
no-boolean-literal-comparison = true
trust-existence-checks = false
```

### Lenient mode

```toml
[analyzer]
check-missing-type-hints = false
strict-list-index-checks = false
strict-array-index-existence = false
no-boolean-literal-comparison = false
enforce-class-finality = false
require-api-or-internal = false
trust-existence-checks = true
check-throws = false
```

When introducing Mago to an existing codebase, start lenient with a [baseline](/fundamentals/baseline/) and tighten the screws as the code improves.

### Notes on individual flags

`trust-existence-checks` decides whether the analyzer narrows on runtime checks. With it on (the default), this is fine:

```php
function process(object $obj): mixed
{
    if (method_exists($obj, 'toArray')) {
        return $obj->toArray();
    }

    return null;
}
```

Turn it off and the call requires an explicit type guarantee instead.

`allow-implicit-pipe-callable-types` skips the closure / arrow-function type-hint checks when the callable is the right operand of `|>`. The pipe's left operand carries enough type information to derive the parameter, so the missing hint is harmless there.

`strict-array-index-existence` aligns the type system with PHP's runtime semantics for missing keys. PHP turns a missing read into `null` and emits an `Undefined array key` warning at runtime; with this flag on, the analyzer emits `possibly-undefined-int-array-index` (or `-string-array-index`) and widens the result to `T|null`, so subsequent `=== null` and `??` checks behave as expected. It applies to `list<T>` reads at non-zero indices, optional entries of `array{...}` shapes, and `array<K, V>` lookups by arbitrary keys. It is opt-in because making it the default is noisy on idiomatic PHP that destructures or indexes lists without first asserting existence.

`allow-possibly-undefined-array-keys = false` is deprecated. It only warned on `array<K, V>` reads with a single literal key and never widened the type to `T|null`, so `=== null` after the read was reported as redundant. Replace it with `strict-array-index-existence = true`, which warns more thoroughly and reflects the runtime semantics in the type. Setting `allow-possibly-undefined-array-keys = false` will emit a deprecation warning on the CLI.

`allow-array-truthy-operand` controls whether arrays are accepted as operands of `&&`, `||`, and `xor`. PHP coerces empty arrays to `false` and non-empty arrays to `true` — the same truthiness used by a standalone `if ($array)`. By default the analyzer flags array operands of logical operators with `invalid-operand` to call out the implicit `bool` coercion; turning the option on suppresses that warning so codebases that rely on the coercion can keep their style. Standalone `if ($array)` is never affected.

## Performance tuning

The analyzer uses internal thresholds to balance depth against speed. Settings live under `[analyzer.performance]`.

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `saturation-complexity-threshold` | `u16` | `8192` | Maximum clauses during CNF saturation. |
| `disjunction-complexity-threshold` | `u16` | `4096` | Maximum clauses per side in OR operations. |
| `negation-complexity-threshold` | `u16` | `4096` | Maximum cumulative complexity when negating formulas. |
| `consensus-limit-threshold` | `u16` | `256` | Upper limit for consensus optimisation passes. |
| `formula-size-threshold` | `u16` | `512` | Maximum logical formula size before simplification is skipped. |
| `string-combination-threshold` | `u16` | `128` | Maximum literal strings tracked before generalising to `string`. |
| `integer-combination-threshold` | `u16` | `128` | Maximum literal integers tracked before generalising to `int`. |
| `array-combination-threshold` | `u16` | `32` | Maximum sealed keyed-array shapes tracked individually before merging. |
| `loop-assignment-depth-threshold` | `u8` | `1` | Maximum loop fixed-point iteration depth. `0` disables re-iteration. |

`string-concat-combination-threshold` is still accepted as an alias for `string-combination-threshold`.

### When to adjust

Defaults work for most projects. Lower the thresholds if analysis feels too slow at the cost of some inference precision. Raise them if you need deeper inference on highly conditional code.

### What each threshold controls

The analyzer turns type constraints into Conjunctive Normal Form (CNF) logical formulas. These can grow exponentially with complex conditions, so the thresholds prevent runaway computation.

- Saturation complexity caps clauses processed during formula simplification. When exceeded, simplification stops early.
- Disjunction complexity bounds clause growth when combining `OR`. Wide unions and many branches can hit this.
- Negation complexity bounds expansion when negating formulas, for example computing `else` branches from a complicated `if`.
- Consensus limit caps an optimisation pass that detects logical tautologies. Higher values may find more simplifications.
- Formula size is the overall complexity ceiling before the analyzer falls back to simpler inference.
- String / integer combination caps how many literal values are tracked before the analyzer widens to `string` or `int`. Without these, very large arrays or switch statements would push combine cost to O(n²).
- Array combination caps how many distinct sealed keyed-array shapes are kept separate during type combination. When procedural code accumulates many slightly different shapes for the same variable across branches, the combiner keeps each one until this threshold is hit and merges them into a generalised shape. Increase it for code that depends on very precise per-key narrowing.
- Loop assignment depth caps how many fixed-point iterations the loop analyzer runs over each loop body. With a chain of `N` loop-carried dependencies, up to `N` extra passes may be needed for types at the end of the chain to fully stabilise; each pass re-analyses the whole body. The default of `1` is enough for almost all real code. Raise it to `2` or `3` for codebases that need very precise narrowing of deep loop-carried chains. `0` disables re-iteration entirely, which is the fastest setting but may leave some self-dependent types wider than necessary.

### Examples

Fast analysis, lower precision:

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

Deep analysis, slower:

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

Raising thresholds can swing analysis time noticeably on codebases with heavy conditional logic or files with thousands of array operations. Test on your project before deploying to CI.

### Diagnosing slow runs

If Mago feels slow, that usually points to a bug in Mago rather than normal behaviour. For reference, on an Apple M1 Pro the analyzer covers all of [`WordPress/wordpress-develop`](https://github.com/WordPress/wordpress-develop) in under two seconds and [`php-standard-library/php-standard-library`](https://github.com/php-standard-library/php-standard-library) in under 200 milliseconds. Numbers will vary with hardware and project size, but as a rough threshold: if Mago takes more than 30 seconds to analyse your project, something is off, either in Mago or in a pathological input it is tripping over.

The same applies to a regression you notice between releases. If a previously fast analysis suddenly becomes slow, that is worth reporting.

Re-run with `MAGO_LOG=trace` to get a full pipeline trace:

```bash
MAGO_LOG=trace mago analyze
```

With tracing on, Mago:

- Starts a hang watcher that flags any single file analysing for more than a few seconds. Useful for catching the file that sends the analyzer into a long or infinite loop.
- Prints the slowest files seen during the parallel analyse phase, so you can see which inputs dominated total time.
- Emits per-phase durations for source discovery, compilation, codebase merge, metadata population, parallel analyse, reduce, and so on, so you can tell which stage is responsible.

When reporting a slow run or a regression, include the full trace output and the file the hang watcher points at. Anonymising names and scrubbing sensitive literals is enough; we just need to reproduce the shape of the input.
