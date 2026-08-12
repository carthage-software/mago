<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\SourceLocation;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ParameterMetadata
{
    /** @param list<AttributeMetadata> $attributes */
    public function __construct(
        public readonly string $name,
        public readonly SourceLocation $location,
        public readonly SourceLocation $nameLocation,
        public readonly ?TypeMetadata $declaredType,
        public readonly ?TypeMetadata $type,
        public readonly ?TypeMetadata $outType,
        public readonly ?TypeMetadata $closureThisType,
        public readonly ?TypeMetadata $defaultType,
        public readonly array $attributes,
        public readonly MetadataFlags $flags,
    ) {}
}
