<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\ExpressionThrowTypeProvider;

/**
 * @internal
 */
final class RegisteredExpressionThrowTypeProvider
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly ExpressionThrowTypeProvider $callback,
    ) {}
}
