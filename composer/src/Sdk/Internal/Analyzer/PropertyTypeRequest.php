<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\PropertyAccess;

/** @internal */
final class PropertyTypeRequest
{
    /**
     * @param list<int<0, 65535>> $providerIndices
     */
    public function __construct(
        public readonly int $generation,
        public readonly array $providerIndices,
        public readonly PropertyAccess $access,
    ) {}
}
