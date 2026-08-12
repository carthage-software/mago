<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\SourceLocation;

/**
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class PropertyMetadata
{
    /**
     * @param array<string, PropertyHookMetadata> $hooks
     * @param list<VersionRange> $availableVersions
     */
    public function __construct(
        public readonly string $name,
        public readonly ?SourceLocation $location,
        public readonly ?SourceLocation $nameLocation,
        public readonly Visibility $readVisibility,
        public readonly Visibility $writeVisibility,
        public readonly ?TypeMetadata $declaredType,
        public readonly ?TypeMetadata $type,
        public readonly ?TypeMetadata $writeType,
        public readonly ?TypeMetadata $defaultType,
        public readonly MetadataFlags $flags,
        public readonly array $hooks,
        public readonly array $availableVersions,
    ) {}
}
