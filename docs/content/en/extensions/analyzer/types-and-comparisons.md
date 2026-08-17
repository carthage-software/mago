+++
title = "Types and comparisons"
description = "Construct complete Mago types and compare them with native codebase semantics."
nav_order = 80
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Types and comparisons

`Mago\Sdk\Analyzer\Type` is an immutable representation of Mago's semantic union type. Types received from Mago retain the complete native atomic structure; extensions can inspect, reuse, combine, and return them without reducing them to strings.

## Common factories

Convenience factories cover frequently returned types:

```php
use Acme\Collection\Collection;
use Acme\Domain\User;
use Mago\Sdk\Analyzer\Type;

Type::mixed();
Type::never();
Type::null();
Type::void();
Type::bool();
Type::true();
Type::false();
Type::int();
Type::literalInt(42);
Type::nonNegativeInt();
Type::float();
Type::string();
Type::literalString('users');
Type::nonEmptyString();
Type::object();
Type::namedObject(Collection::class, Type::namedObject(User::class));
Type::array(Type::string(), Type::mixed());
Type::list(Type::namedObject(User::class));
Type::union(Type::null(), Type::namedObject(User::class));
```

`namedObject()` accepts generic parameters. `union()` requires at least two members. Use `fromAtomic()` or `fromAtomics()` when constructing richer types directly, and `withFlags()` to preserve or replace union-level `TypeFlags`.

`getLiteralInt()`, `getLiteralString()`, `getLiteralClassString()`, and `getLiteralBool()` return a value only when the union represents exactly that literal. Casting a type to string is useful for diagnostics and debugging, not semantic comparison.

## Complete atomic representation

`Type::$atomicTypes` exposes the complete union members. The public atomic DTOs under `Mago\Sdk\Analyzer\Type` cover:

- aliases and references;
- mixed, scalar, resource, never, void, null, and placeholder atomics;
- integer ranges and literals;
- refined strings, class-string variants, and literal strings;
- named objects, enums, any-object, object shapes, and objects with required methods or properties;
- arrays with explicit keyed items and fallback parameters;
- lists with element and length constraints;
- iterables and callables with signatures and constraints;
- generic parameters and their defining parent;
- conditional and variable-dependent types;
- derived key-of, value-of, integer-mask, properties-of, indexed-access, constructor, template, and intersection forms.

Supporting DTOs retain array keys and items, callable parameters and signatures, object properties, generic variance, visibility, and type flags. This mirrors the information available in Mago's native `TUnion` and atomic types rather than offering a lossy extension-only subset.

When Mago supplies a type, prefer transforming its existing atomic DTOs over rebuilding it from a printed description.

## Native comparisons

PHP code should not implement its own subtype or overlap algorithm. Every semantic context exposes a `TypeComparator` backed by Mago's native codebase-aware type system:

```php
if ($context->types->equals($actual, $expected)) {
    // Exactly equivalent.
}

if ($context->types->isContainedBy($actual, $expected)) {
    // Every value in $actual is accepted by $expected.
}

if ($context->types->canBeIdentical($left, $right)) {
    // The two types have at least one identity-compatible possibility.
}
```

Comparisons account for class inheritance, interfaces, generics, shapes, refinements, and other native analyzer rules. `compareMultiple()` reliably caches both true and false outcomes and deduplicates repeated keys within a batch.

## Batch comparisons

Use `compareMultiple()` when a callback needs more than one relationship:

```php
use Mago\Sdk\Analyzer\TypeComparison;

[$accepts, $overlaps, $same] = $context->types->compareMultiple([
    TypeComparison::containedBy($input, $parameter),
    TypeComparison::canBeIdentical($left, $right),
    TypeComparison::equal($inferred, $declared),
]);
```

The SDK deduplicates and caches comparisons, then sends unresolved work in one nested request. A batch may contain at most 65,536 comparisons and returns booleans in input order.

Batching is especially important in loops over metadata. Do not make one IPC request per method, property, or candidate type.

## Request references

Types decoded from a provider request may internally reference the native request's type table. They remain safe to inspect, compare, and return during that callback. The SDK tracks request-backed values and uses the correct cache scope automatically.

Do not serialize internal encodings yourself or retain protocol handles as a durable on-disk format. The documented API is the atomic DTO graph and `Type` factories.

## Type flags

`TypeFlags` preserves union-wide facts that are not individual atomic members. When modifying a received type, decide explicitly whether those flags remain valid. `Type::withFlags()` returns a new value; it does not mutate the original.

## Performance rules

- Reuse types supplied in the callback instead of reconstructing equivalents.
- Check literal getters before invoking a native comparison.
- Batch independent comparisons.
- Do not compare string renderings.
- Avoid expanding every atomic detail unless the provider needs it.
- Reuse the context comparator so its supported generation-local caches can apply.
