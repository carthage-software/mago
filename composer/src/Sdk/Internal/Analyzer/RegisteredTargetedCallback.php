<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\FileAnalysisRequirement;

/**
 * @template TCallback of object
 * @template TTarget
 * @internal
 */
final class RegisteredTargetedCallback
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     * @param TCallback $callback
     * @param non-empty-list<TTarget> $targets
     * @param list<FileAnalysisRequirement> $requirements
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly object $callback,
        public readonly array $targets,
        public readonly array $requirements = [],
    ) {}
}
