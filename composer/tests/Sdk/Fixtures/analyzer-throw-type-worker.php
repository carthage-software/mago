<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\ExpressionThrowTypeProvider;
use Mago\Sdk\Analyzer\ExpressionThrowTypeProviderContext;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\FunctionThrowTypeProvider;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\MethodThrowTypeProvider;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\ThrowTypeProviderContext;
use Mago\Sdk\Extension;
use Mago\Sdk\Worker;

require_once \dirname(__DIR__, 4) . '/vendor/autoload.php';

final class ThrowTypePlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            'throw-types-test',
            'Throw type test',
            'Integration-test providers.',
            defaultEnabled: true,
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerFunctionThrowTypeProvider(new class implements FunctionThrowTypeProvider {
            public function getTargets(): array
            {
                return [
                    FunctionTarget::exact('external_throw'),
                    FunctionTarget::exact('external_empty'),
                    FunctionTarget::exact('external_fallback'),
                    FunctionTarget::exact('external_caught'),
                ];
            }

            public function getThrownExceptions(ThrowTypeProviderContext $context): ?array
            {
                return match ($context->invocation->name) {
                    'external_throw' => ['\\RuntimeException'],
                    'external_empty' => [],
                    'external_caught' => ['\\Throwable'],
                    default => null,
                };
            }
        });
        $registry->registerMethodThrowTypeProvider(new class implements MethodThrowTypeProvider {
            public function getTargets(): array
            {
                return [MethodTarget::exact('ThrowSubject', 'run')];
            }

            public function getThrownExceptions(ThrowTypeProviderContext $context): ?array
            {
                return ['\\RuntimeException'];
            }
        });
        $registry->registerExpressionThrowTypeProvider(new class implements ExpressionThrowTypeProvider {
            public function getThrownExceptions(ExpressionThrowTypeProviderContext $context): ?array
            {
                return $context->expression === '1 + 1' ? ['\\DomainException'] : null;
            }
        });
    }
}

/** @mago-expect lint:single-class-per-file */
final class DisabledThrowTypePlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            'disabled-throw-types-test',
            'Disabled throw type test',
            'Disabled integration providers.',
            defaultEnabled: false,
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerFunctionThrowTypeProvider(new class implements FunctionThrowTypeProvider {
            public function getTargets(): array
            {
                return [FunctionTarget::exact('external_disabled')];
            }

            public function getThrownExceptions(ThrowTypeProviderContext $context): ?array
            {
                return ['\\RuntimeException'];
            }
        });
        $registry->registerMethodThrowTypeProvider(new class implements MethodThrowTypeProvider {
            public function getTargets(): array
            {
                return [MethodTarget::exact('ThrowSubject', 'disabledRun')];
            }

            public function getThrownExceptions(ThrowTypeProviderContext $context): ?array
            {
                return ['\\RuntimeException'];
            }
        });
        $registry->registerExpressionThrowTypeProvider(new class implements ExpressionThrowTypeProvider {
            public function getThrownExceptions(ExpressionThrowTypeProviderContext $context): ?array
            {
                return $context->expression === '2 + 2' ? ['\\RuntimeException'] : null;
            }
        });
    }
}

(new Worker(new Extension(
    identifier: 'mago/sdk-throw-type-test',
    name: 'Mago SDK throw type test',
    version: '1.0.0',
    analyzerPlugins: [new ThrowTypePlugin(), new DisabledThrowTypePlugin()],
)))->run();
