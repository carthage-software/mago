<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\SourceLocation;

/**
 * A complete stable snapshot of a function, method, or closure signature.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class FunctionLikeMetadata
{
    /**
     * @param list<ParameterMetadata> $parameters
     * @param list<TemplateMetadata> $templates
     * @param list<AttributeMetadata> $attributes
     * @param list<TypeMetadata> $thrownTypes
     * @param list<string> $globalsAccessed
     * @param array<string, TypeMetadata> $whereConstraints
     * @param list<VersionRange> $availableVersions
     */
    public function __construct(
        public readonly FunctionLikeKind $kind,
        public readonly string $name,
        public readonly string $originalName,
        public readonly SourceLocation $location,
        public readonly ?SourceLocation $nameLocation,
        public readonly array $parameters,
        public readonly ?TypeMetadata $declaredReturnType,
        public readonly ?TypeMetadata $returnType,
        public readonly array $templates,
        public readonly array $attributes,
        public readonly array $thrownTypes,
        public readonly array $globalsAccessed,
        public readonly bool $hasDocblock,
        public readonly MetadataFlags $flags,
        public readonly array $availableVersions,
        public readonly ?Visibility $visibility,
        public readonly bool $final,
        public readonly bool $abstract,
        public readonly bool $static,
        public readonly bool $constructor,
        public readonly array $whereConstraints,
    ) {}
}
