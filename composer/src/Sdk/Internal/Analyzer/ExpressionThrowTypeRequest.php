<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Span;

/** @internal */
final class ExpressionThrowTypeRequest
{
    /**
     * @param list<int<0, 65535>> $providerIndices
     * @mago-expect lint:excessive-parameter-list
     */
    public function __construct(
        public readonly int $generation,
        public readonly array $providerIndices,
        public readonly string $sourceFilePath,
        public readonly Span $span,
        public readonly string $expression,
        public readonly ?Type $expressionType,
    ) {}
}
