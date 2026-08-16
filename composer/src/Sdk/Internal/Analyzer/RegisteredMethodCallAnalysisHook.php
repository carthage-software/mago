<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\FileAnalysisRequirement;
use Mago\Sdk\Analyzer\MethodCallAnalysisHook;
use Mago\Sdk\Analyzer\MethodTarget;

/**
 * @internal
 */
final class RegisteredMethodCallAnalysisHook
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     * @param non-empty-list<MethodTarget> $targets
     * @param list<FileAnalysisRequirement> $requirements
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly MethodCallAnalysisHook $hook,
        public readonly array $targets,
        public readonly array $requirements,
    ) {}
}
