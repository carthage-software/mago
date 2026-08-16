<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\FunctionAssertionProvider;
use Mago\Sdk\Analyzer\FunctionTarget;

/** @internal */
final class RegisteredFunctionAssertionProvider
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     * @param non-empty-list<FunctionTarget> $targets
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly FunctionAssertionProvider $provider,
        public readonly array $targets,
    ) {}
}
