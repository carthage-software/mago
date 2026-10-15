+++
title = "Throw type providers"
description = "Describe exceptions thrown by calls and expressions in PHP extensions."
nav_order = 35
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Throw type providers

Throw type providers let an analyzer extension describe exceptions that Mago cannot infer from native declarations. They run when throw checking is enabled.

## Function and method calls

Implement `FunctionThrowTypeProvider` or `MethodThrowTypeProvider` and register the provider with the analyzer plugin registry:

```php
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\FunctionThrowTypeProvider;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\ThrowTypeProviderContext;

final class DecodePayloadThrows implements FunctionThrowTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('Acme\\decode_payload')];
    }

    public function getThrownExceptions(ThrowTypeProviderContext $context): ?array
    {
        $format = $context->invocation->getArgument(0)?->type?->getLiteralString();

        return $format === 'json' ? ['\\JsonException'] : null;
    }
}

// In AnalyzerPlugin::register():
$registry->registerFunctionThrowTypeProvider(new DecodePayloadThrows());
```

`ThrowTypeProviderContext` exposes the configured PHP version, read-only `Codebase`, `Invocation`, `TypeComparator`, and cancellation token. Invocation arguments include their source expressions and inferred types.

Method providers use `MethodTarget` and receive the same context. Targets can be exact or patterned in the same way as return-type providers.

Return `null` to let the next matching provider or native analysis handle the call. Return an empty list when the call does not throw. A non-empty list supplies the complete exception contract for that call and takes precedence over native `@throws` declarations and other throw providers. Use fully qualified exception class names.

## Other expressions

Implement `ExpressionThrowTypeProvider` for exceptions from expressions such as property access, array access, or framework-specific syntax:

```php
use Mago\Sdk\Analyzer\ExpressionThrowTypeProvider;
use Mago\Sdk\Analyzer\ExpressionThrowTypeProviderContext;

final class ArrayAccessThrows implements ExpressionThrowTypeProvider
{
    public function getThrownExceptions(ExpressionThrowTypeProviderContext $context): ?array
    {
        if ($context->expression === '$value[0]') {
            return null;
        }

        return ['\\DomainException'];
    }
}
```

Register it with `PluginRegistry::registerExpressionThrowTypeProvider()`. The context includes the source file path, expression span and text, optional inferred type, codebase, type comparator, PHP version, and cancellation token. Return `null` to continue to the next provider or native analysis; an empty list means the expression does not throw.

Provider failures are treated as unavailable hints. Mago logs the failure and falls back to native throw analysis.
