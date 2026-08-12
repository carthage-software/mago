<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\AfterAnalysisHook;
use Mago\Sdk\Analyzer\AfterFileAnalysisHook;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;

/**
 * @internal
 */
final class RegisteredPlugin
{
    /**
     * @param non-empty-string $extension
     * @param list<RegisteredFunctionReturnTypeProvider> $functionProviders
     * @param list<RegisteredMethodReturnTypeProvider> $methodProviders
     * @param list<BeforeAnalysisHook> $beforeAnalysisHooks
     * @param list<AfterFileAnalysisHook> $afterFileAnalysisHooks
     * @param list<AfterAnalysisHook> $afterAnalysisHooks
     */
    public function __construct(
        public readonly int $index,
        public readonly string $extension,
        public readonly Plugin $plugin,
        public readonly PluginDefinition $definition,
        public readonly array $functionProviders,
        public readonly array $methodProviders,
        public readonly array $beforeAnalysisHooks,
        public readonly array $afterFileAnalysisHooks,
        public readonly array $afterAnalysisHooks,
    ) {}
}
