<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\ExpressionThrowTypeProvider;
use Mago\Sdk\Analyzer\ExpressionThrowTypeProviderContext;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\FunctionThrowTypeProvider;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\MethodThrowTypeProvider;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\ThrowTypeProviderContext;
use PHPUnit\Framework\TestCase;

final class ThrowTypeProviderRegistryTest extends TestCase
{
    public function testThrowTypeProvidersPreserveRegistrationOrder(): void
    {
        $registry = new PluginRegistry();
        $function = new class implements FunctionThrowTypeProvider {
            public function getTargets(): array
            {
                return [FunctionTarget::exact('first')];
            }

            public function getThrownExceptions(ThrowTypeProviderContext $context): ?array
            {
                return null;
            }
        };
        $method = new class implements MethodThrowTypeProvider {
            public function getTargets(): array
            {
                return [MethodTarget::exact('Example', 'run')];
            }

            public function getThrownExceptions(ThrowTypeProviderContext $context): ?array
            {
                return null;
            }
        };
        $expression = new class implements ExpressionThrowTypeProvider {
            public function getThrownExceptions(ExpressionThrowTypeProviderContext $context): ?array
            {
                return null;
            }
        };

        $registry->registerFunctionThrowTypeProvider($function);
        $registry->registerMethodThrowTypeProvider($method);
        $registry->registerExpressionThrowTypeProvider($expression);

        self::assertSame([$function], $registry->getFunctionThrowTypeProviders());
        self::assertSame([$method], $registry->getMethodThrowTypeProviders());
        self::assertSame([$expression], $registry->getExpressionThrowTypeProviders());
    }
}
