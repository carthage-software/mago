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
     */
    public function __construct(
        public readonly ?Type $targetType,
        public readonly ?Type $receiverType,
        public readonly array $argumentTypes,
    ) {}
}
