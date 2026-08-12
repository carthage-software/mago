<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\BeforeAnalysisContext;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\Definition\ClassLikeDefinition;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Extension;
use Mago\Sdk\Worker;
use RuntimeException;

use function dirname;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

final class FailingMutationHook implements BeforeAnalysisHook
{
    public function beforeAnalysis(BeforeAnalysisContext $context): void
    {
        $context->codebase->insertClassLike(new ClassLikeDefinition('MustRollBack'));
        throw new RuntimeException('Deliberate transaction failure.');
    }
}

/** @mago-expect lint:single-class-per-file */
final class FailingMutationPlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition('rollback', 'Rollback proof', 'Proves failed metadata transactions are discarded.');
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerBeforeAnalysisHook(new FailingMutationHook());
    }
}

(new Worker(new Extension(
    identifier: 'mago/mutation-rollback-proof',
    name: 'Mago analyzer mutation rollback proof',
    version: '1.0.0',
    analyzerPlugins: [new FailingMutationPlugin()],
)))->run();
