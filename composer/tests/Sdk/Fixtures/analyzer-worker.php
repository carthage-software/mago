<?php

declare(strict_types=1);

use Mago\Sdk\Analyzer\FunctionReturnTypeProvider;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\MethodReturnTypeProvider;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\ReturnTypeProviderContext;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Extension;
use Mago\Sdk\Worker;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

final class DemoServiceProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('demo_service')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        return Type::namedObject('DemoService');
    }
}

/** @mago-expect lint:single-class-per-file */
final class DemoIdentityProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('demo_identity')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        return $context->invocation->getArgument(0, 'value')?->type;
    }
}

/** @mago-expect lint:single-class-per-file */
final class DemoFactoryProvider implements MethodReturnTypeProvider
{
    public function getTargets(): array
    {
        return [MethodTarget::exact('DemoFactory', 'create')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        return Type::namedObject('DemoService');
    }
}

/** @mago-expect lint:single-class-per-file */
final class DemoAnalyzerPlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            identifier: 'demo-analyzer',
            name: 'Demo analyzer',
            description: 'Exercises external function and method return-type providers.',
            aliases: ['demo'],
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerFunctionReturnTypeProvider(new DemoServiceProvider());
        $registry->registerFunctionReturnTypeProvider(new DemoIdentityProvider());
        $registry->registerMethodReturnTypeProvider(new DemoFactoryProvider());
    }
}

$extension = new Extension(
    identifier: 'mago/demo-extension',
    name: 'Mago analyzer extension fixture',
    version: '1.0.0',
    analyzerPlugins: [new DemoAnalyzerPlugin()],
);

(new Worker($extension))->run();
