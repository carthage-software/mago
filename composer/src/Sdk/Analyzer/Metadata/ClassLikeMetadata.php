<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\SourceLocation;

/**
 * A stable semantic snapshot of a class, interface, trait, or enum.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ClassLikeMetadata
{
    /**
     * @param list<string> $directParentInterfaces
     * @param list<string> $parentInterfaces
     * @param list<string> $parentClasses
     * @param list<string> $requiredExtends
     * @param list<string> $requiredImplements
     * @param list<string> $usedTraits
     * @param list<string> $unresolvedHierarchyDependencies
     * @param list<string> $methods
     * @param list<string> $pseudoMethods
     * @param list<string> $staticPseudoMethods
     * @param list<string> $properties
     * @param list<string> $magicProperties
     * @param list<string> $constants
     * @param list<string> $enumCases
     * @param list<string>|null $children
     * @param list<string>|null $permittedInheritors
     * @param list<TemplateMetadata> $templates
     * @param list<AttributeMetadata> $attributes
     * @param array<string, TypeMetadata> $typeAliases
     * @param list<Type> $mixins
     * @param list<VersionRange> $availableVersions
     */
    public function __construct(
        public readonly string $name,
        public readonly string $originalName,
        public readonly ClassLikeKind $kind,
        public readonly SourceLocation $location,
        public readonly ?SourceLocation $nameLocation,
        public readonly MetadataFlags $flags,
        public readonly ?string $directParentClass,
        public readonly array $directParentInterfaces,
        public readonly array $parentInterfaces,
        public readonly array $parentClasses,
        public readonly array $requiredExtends,
        public readonly array $requiredImplements,
        public readonly array $usedTraits,
        public readonly array $unresolvedHierarchyDependencies,
        public readonly array $methods,
        public readonly array $pseudoMethods,
        public readonly array $staticPseudoMethods,
        public readonly array $properties,
        public readonly array $magicProperties,
        public readonly array $constants,
        public readonly array $enumCases,
        public readonly ?array $children,
        public readonly ?array $permittedInheritors,
        public readonly array $templates,
        public readonly array $attributes,
        public readonly array $typeAliases,
        public readonly array $mixins,
        public readonly ?Type $enumType,
        public readonly ?bool $sealedMethods,
        public readonly ?bool $sealedProperties,
        public readonly array $availableVersions,
    ) {}

    /**
     * Returns whether inherited metadata is incomplete because a hierarchy dependency could not be resolved.
     */
    public function hasIncompleteHierarchy(): bool
    {
        return $this->unresolvedHierarchyDependencies !== [];
    }
}
