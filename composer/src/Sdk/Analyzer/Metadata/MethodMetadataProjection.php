<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Assertion\Assertion;
use Mago\Sdk\Analyzer\Type\FunctionLikeIdentifier;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\SourceLocation;

/**
 * A field-selected view of one visible method and its declaration.
 *
 * `$method` identifies the method as visible on the matched class. `$identifier`
 * identifies its declaration. Both are always populated. Every optional property
 * is `null` when its field group was not requested; use {@see has()} to distinguish
 * an omitted group from a selected nullable value.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class MethodMetadataProjection
{
    /**
     * @param list<ParameterMetadata>|null $parameters
     * @param list<TemplateMetadata>|null $templates
     * @param list<AttributeMetadata>|null $attributes
     * @param list<TypeMetadata>|null $thrownTypes
     * @param array<string, list<Assertion>>|null $assertions
     * @param array<string, list<Assertion>>|null $ifTrueAssertions
     * @param array<string, list<Assertion>>|null $ifFalseAssertions
     * @param list<string>|null $globalsAccessed
     * @param list<VersionRange>|null $availableVersions
     * @param array<string, TypeMetadata>|null $whereConstraints
     *
     * @internal
     */
    public function __construct(
        public readonly MemberIdentifier $method,
        public readonly FunctionLikeIdentifier $identifier,
        public readonly int $fields,
        public readonly ?string $name,
        public readonly ?string $originalName,
        public readonly ?SourceLocation $location,
        public readonly ?SourceLocation $nameLocation,
        public readonly ?array $parameters,
        public readonly ?TypeMetadata $declaredReturnType,
        public readonly ?TypeMetadata $returnType,
        public readonly ?array $templates,
        public readonly ?array $attributes,
        public readonly ?array $thrownTypes,
        public readonly ?array $assertions,
        public readonly ?array $ifTrueAssertions,
        public readonly ?array $ifFalseAssertions,
        public readonly ?bool $assertionsInferred,
        public readonly ?array $globalsAccessed,
        public readonly ?bool $hasDocblock,
        public readonly ?MetadataFlags $flags,
        public readonly ?array $availableVersions,
        public readonly ?Visibility $visibility,
        public readonly ?bool $final,
        public readonly ?bool $abstract,
        public readonly ?bool $static,
        public readonly ?bool $constructor,
        public readonly ?array $whereConstraints,
    ) {}

    public function has(int $fields): bool
    {
        return ($this->fields & $fields) === $fields;
    }
}
