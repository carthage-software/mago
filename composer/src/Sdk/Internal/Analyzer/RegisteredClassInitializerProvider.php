<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\ClassInitializerProvider;
use Mago\Sdk\Analyzer\ClassTarget;

/** @internal */
final class RegisteredClassInitializerProvider
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     * @param non-empty-list<ClassTarget> $targets
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly ClassInitializerProvider $provider,
        public readonly array $targets,
    ) {}
}
