<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\SourceLocation;

/**
 * A field-selected view of property metadata.
 *
 * Results correspond to the requested member identifiers in input order. Every
 * metadata property is null when its field group was not requested; use has()
 * to distinguish an omitted group from a selected nullable value.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class PropertyMetadataProjection
{
    /**
     * @param list<AttributeMetadata>|null $attributes
     * @param array<string, PropertyHookMetadata>|null $hooks
     * @param list<VersionRange>|null $availableVersions
     *
     * @internal
     */
    public function __construct(
        public readonly int $fields,
        public readonly ?string $name,
        public readonly ?SourceLocation $location,
        public readonly ?SourceLocation $nameLocation,
        public readonly ?Visibility $readVisibility,
        public readonly ?Visibility $writeVisibility,
        public readonly ?TypeMetadata $declaredType,
        public readonly ?TypeMetadata $type,
        public readonly ?TypeMetadata $writeType,
        public readonly ?TypeMetadata $defaultType,
        public readonly ?array $attributes,
        public readonly ?MetadataFlags $flags,
        public readonly ?array $hooks,
        public readonly ?array $availableVersions,
    ) {}

    public function has(int $fields): bool
    {
        return ($this->fields & $fields) === $fields;
    }
}
