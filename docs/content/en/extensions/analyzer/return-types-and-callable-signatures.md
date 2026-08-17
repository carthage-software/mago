+++
title = "Return types and callable signatures"
description = "Describe dynamic functions, methods, arguments, and return types."
nav_order = 30
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Return types and callable signatures

Return-type providers refine the result of selected calls after Mago has analyzed their arguments. Callable-signature providers describe the parameters accepted by dynamic callables before argument analysis begins. A framework API often needs both.

Place callable providers under `src/Mago/Analyzer/Providers/`.

## Target a callable

Function providers return `FunctionTarget` values:

```php
use Mago\Sdk\Analyzer\FunctionTarget;

return [
    FunctionTarget::exact('collect'),
    FunctionTarget::prefix('Acme\\Support\\'),
    FunctionTarget::namespace('Acme\\Helpers'),
];
```

Method providers return `MethodTarget` values:

```php
use Acme\Framework\Container;
use Acme\Framework\DynamicProxy;
use Mago\Sdk\Analyzer\MethodTarget;

return [
    MethodTarget::exact(Container::class, 'get'),
    MethodTarget::allMethods(DynamicProxy::class),
    MethodTarget::anyClass('macro'),
];
```

An exact class also matches subclasses and interface implementations. `*` patterns are terminal prefix matches; prefer exact targets whenever possible.

`FunctionTarget::namespace('Acme\\Helpers')` is normalized to the prefix `Acme\\Helpers\\`, so it matches functions in that namespace and nested namespaces without matching names such as `Acme\\HelpersExtra\\run`.

## Return-type providers

Implement `FunctionReturnTypeProvider` or `MethodReturnTypeProvider`:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer\Providers;

use Acme\Framework\Cache;
use Acme\Framework\Container;
use Acme\Framework\Logger;
use Mago\Sdk\Analyzer\MethodReturnTypeProvider;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\ReturnTypeProviderContext;
use Mago\Sdk\Analyzer\Type;

final class ContainerReturnTypes implements MethodReturnTypeProvider
{
    public function getTargets(): array
    {
        return [MethodTarget::exact(Container::class, 'get')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $id = $context->invocation->getArgument(0, '$id');
        $service = $id?->type?->getLiteralString();

        return match ($service) {
            'logger' => Type::namedObject(Logger::class),
            'cache' => Type::namedObject(Cache::class),
            default => null,
        };
    }
}
```

Returning `null` delegates to the next matching provider and ultimately preserves Mago's native result, including a declared return type. A non-null result replaces that native return result. A provider should return a type only when it has enough information to improve Mago's answer.

`ReturnTypeProviderContext` provides the PHP version, read-only codebase, invocation, native type comparator, and cancellation token.

## Invocation data

`Invocation` describes the call:

| Member | Meaning |
| :--- | :--- |
| `kind` | `Function`, `InstanceMethod`, or `StaticMethod` |
| `name` | Resolved function or method name |
| `declaringClass` | Resolved declaring class for methods; `null` for functions |
| `receiverType` | Inferred method receiver type; `null` for functions |
| `span` | Complete call span |
| `arguments` | Arguments in source order |

Each `Argument` contains its optional name, unpacked and placeholder flags, source span, exact expression text, and inferred type. `Invocation::getArgument($index, ...$names)` first accepts an unnamed positional argument at the requested index, then searches named arguments by parameter name.

Use the inferred `Argument::$type` for semantics. Expression text is useful for constructs whose widened type loses a source-level constant expression, but it should not be reparsed as a substitute for Mago's type system.

## Callable signatures

A registered function or method return-type provider may additionally implement `CallableSignatureProvider`. Mago asks for a signature before analyzing arguments when it cannot resolve a declared callable:

```php
use Mago\Sdk\Analyzer\CallableSignatureProvider;
use Mago\Sdk\Analyzer\CallableSignatureProviderContext;
use Mago\Sdk\Analyzer\EffectiveCallableSignature;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\CallableParameter;

public function getCallableSignature(
    CallableSignatureProviderContext $context,
): ?EffectiveCallableSignature {
    if ($context->invocation->name !== 'where') {
        return null;
    }

    return new EffectiveCallableSignature([
        new CallableParameter(name: '$column', type: Type::string()),
        new CallableParameter(name: '$value', type: Type::mixed()),
    ]);
}
```

At this stage argument expression types are intentionally `null`: template inference, closure typing, argument counting, and validation have not happened yet. Argument source text, names, flags, and spans are available.

`EffectiveCallableSignature` accepts an ordered list of `CallableParameter` values and an `allowsNamedArguments` flag. A parameter can define:

- an optional `$name`;
- an optional input `type`;
- an optional `closureThisType` for closures passed to that parameter;
- by-reference, variadic, and default-value flags.

Names must be valid PHP variable names. Parameter names must be unique, a variadic parameter must be last and cannot have a default, and required parameters cannot follow optional ones.

A non-null effective signature establishes an otherwise unresolved function or method as a valid callable. The ordinary return-type provider then runs after arguments have been analyzed, so it can use the types inferred from that signature.

## Refining declared signatures

Ordinary `CallableSignatureProvider` callbacks run only for unresolved callables. Also implement `CallableSignatureOverride` when framework behavior must replace the parameters of an existing function, method, constructor, attribute constructor, or partial application.

Overrides are powerful: Mago uses the effective signature for template inference, closure typing, argument count, named arguments, and argument validation. Return `null` unless the override is unquestionably applicable.

## Unresolved-only return types

Implement the marker interface `UndeclaredReturnTypeProvider` when a return provider only handles dynamic callables. Mago skips it when native metadata already declares the function or method, avoiding unnecessary extension requests.

## Type comparison and batching

Use `$context->types` instead of approximating assignability in PHP. When one callback needs several comparisons, use `compareMultiple()` so Mago can answer them in one nested protocol request. See [Types and comparisons](/extensions/analyzer/types-and-comparisons/).

## Failure and fallback

Return types and callable signatures are optional semantic hints. Provider failures are traced and Mago falls back to native declarations or unresolved-call behavior. A provider must therefore avoid reporting its own transport failures as source-code issues.
