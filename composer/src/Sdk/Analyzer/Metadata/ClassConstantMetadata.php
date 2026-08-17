<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\SourceLocation;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ClassConstantMetadata
{
    /**
     * @param list<AttributeMetadata> $attributes
     * @param list<VersionRange> $availableVersions
     */
    public function __construct(
        public readonly string $name,
        public readonly SourceLocation $location,
        public readonly Visibility $visibility,
        public readonly ?TypeMetadata $declaredType,
        public readonly ?TypeMetadata $type,
        public readonly ?Type $inferredType,
        public readonly array $attributes,
        public readonly MetadataFlags $flags,
        public readonly array $availableVersions,
    ) {}
}
