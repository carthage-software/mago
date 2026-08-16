<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\MethodAssertionProvider;
use Mago\Sdk\Analyzer\MethodTarget;

/** @internal */
final class RegisteredMethodAssertionProvider
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     * @param non-empty-list<MethodTarget> $targets
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly MethodAssertionProvider $provider,
        public readonly array $targets,
    ) {}
}
