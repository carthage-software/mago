<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\ClassLikeAnalysisHook;
use Mago\Sdk\Analyzer\ClassLikeTarget;
use Mago\Sdk\Analyzer\FileAnalysisRequirement;

/**
 * @internal
 */
final class RegisteredClassLikeAnalysisHook
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     * @param non-empty-list<ClassLikeTarget> $targets
     * @param list<FileAnalysisRequirement> $requirements
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly ClassLikeAnalysisHook $hook,
        public readonly array $targets,
        public readonly array $requirements,
    ) {}
}
