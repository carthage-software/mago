<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Type;

/**
 * @internal
 */
final class NodeAnalysisData
{
    /**
     * @param list<Type|null> $argumentTypes
     * @param list<int<0, 65535>> $methodCallHookIndices
     */
    public function __construct(
        public readonly ?Type $targetType,
        public readonly ?Type $receiverType,
        public readonly array $argumentTypes,
        public readonly array $methodCallHookIndices,
    ) {}
}
