<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\SourceLocation;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class EnumCaseMetadata
{
    /**
     * @param list<AttributeMetadata> $attributes
     * @param list<VersionRange> $availableVersions
     */
    public function __construct(
        public readonly string $name,
        public readonly SourceLocation $location,
        public readonly SourceLocation $nameLocation,
        public readonly ?Type $valueType,
        public readonly array $attributes,
        public readonly MetadataFlags $flags,
        public readonly array $availableVersions,
    ) {}
}
