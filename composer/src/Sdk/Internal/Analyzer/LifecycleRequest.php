<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\FileAnalysis;
use Mago\Sdk\Analyzer\ProjectAnalysis;

/**
 * @internal
 */
final class LifecycleRequest
{
    /**
     * @param non-empty-list<int<0, 65535>> $pluginIndices
     * @param FileAnalysis|ProjectAnalysis|non-empty-list<FileAnalysis>|null $analysis
     */
    public function __construct(
        public readonly int $generation,
        public readonly array $pluginIndices,
        public readonly FileAnalysis|ProjectAnalysis|array|null $analysis,
    ) {}
}
