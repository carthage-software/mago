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

use function dirname;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

final class ConflictingMutationHook implements BeforeAnalysisHook
{
    public function beforeAnalysis(BeforeAnalysisContext $context): void
    {
        $context->codebase->insertClassLike(new ClassLikeDefinition('ContendedSymbol'));
    }
}

/** @mago-expect lint:single-class-per-file */
final class ConflictingMutationPlugin implements Plugin
{
    public function __construct(
        private readonly string $identifier,
    ) {}

    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition($this->identifier, $this->identifier, 'Exercises metadata ownership conflicts.');
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerBeforeAnalysisHook(new ConflictingMutationHook());
    }
}

(new Worker(new Extension(
    identifier: 'mago/mutation-conflict-proof',
    name: 'Mago analyzer mutation conflict proof',
    version: '1.0.0',
    analyzerPlugins: [
        new ConflictingMutationPlugin('conflict-one'),
        new ConflictingMutationPlugin('conflict-two'),
    ],
)))->run();
