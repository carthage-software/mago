+++
title = "Analyzer"
description = "What the analyzer does, how it differs from the linter, and where to read next."
nav_order = 10
nav_section = "Tools"
nav_subsection = "Analyzer"
+++
# Analyzer

A static analysis engine for PHP. It builds a semantic model of the entire project, then walks every function, method, and expression to catch type errors and logical impossibilities before they ever run.

## Analyzer vs linter

Both tools find issues, but they operate at different levels.

The **linter** looks at the *shape* of code: stylistic issues, inconsistencies, code smells. It does not need to know what the code does at runtime.

The **analyzer** looks at the *meaning* of code. It tracks the type of every variable through every branch, knows what each method on a class actually returns, and follows what exceptions can propagate. It catches impossibilities like calling a method that does not exist on the type at hand, passing a `?Order` where `Order` is required, or returning `null` from a function annotated as never returning null.

If your code were an essay, the linter is the grammar pass and the analyzer is the fact-check.

## What the analyzer offers

- **Type inference.** The analyzer understands the type of every expression even when type hints are partial. It supports Psalm and PHPStan annotations, generics, conditional types, and flow narrowing.
- **Whole-program awareness.** Analysis runs across the project, so calls into other files surface real signature mismatches.
- **Speed.** Rust core, parallelised, runs an entire project in seconds.
- **Heuristic checks.** A configurable set of advisory checks for code-quality concerns that are not strict errors but often indicate latent bugs.

## Callable parameter timing

Use `@param-immediately-invoked-callable` and `@param-later-invoked-callable` to describe when a callable parameter runs:

```php
/**
 * @param-immediately-invoked-callable $callback
 */
function run(callable $callback): void
{
    $callback();
}

/**
 * @param-later-invoked-callable $callback
 */
function register(callable $callback): void
{
    // The callback may run after this function returns.
}
```

The tag must name an existing callable parameter or a parameter whose array values are callables. Unknown names, non-callable types, and conflicting timing tags are reported as `invalid-param-tag`. Built-in callback parameters use the same metadata as userland functions.

Immediate timing does not promise that callback exceptions escape the wrapper. With `check-throws` enabled, Mago propagates callback exceptions for annotated built-ins, including exceptions inferred from inline callback bodies. For userland wrappers, callback throws remain checked at the callback; the wrapper's own `@throws` contract determines what callers must handle, because the analyzer does not infer catch behavior across calls. A callback can therefore still get a closure-local warning even when the wrapper catches it. Later-invoked callbacks are not propagated to the registration call.

## Where to next

- [Command reference](/tools/analyzer/command-reference/): every flag `mago analyze` accepts.
- [Configuration reference](/tools/analyzer/configuration-reference/): every option Mago accepts under `[analyzer]`.
