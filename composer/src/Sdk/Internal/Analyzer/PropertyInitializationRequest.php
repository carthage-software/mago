<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Metadata\PropertyMetadata;

/** @internal */
final class PropertyInitializationRequest
{
    /**
     * @param list<int<0, 65535>> $providerIndices
     * @param non-empty-string $declaringClass
     */
    public function __construct(
        public readonly int $generation,
        public readonly array $providerIndices,
        public readonly string $declaringClass,
        public readonly PropertyMetadata $property,
    ) {}
}
