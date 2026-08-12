<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Definition;

use Mago\Sdk\Analyzer\Metadata\ClassLikeKind;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * A class-like declaration contributed directly by an extension.
 *
 * Resolved inheritance maps remain owned by Mago and are rebuilt after each
 * accepted mutation batch.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ClassLikeDefinition
{
    /**
     * @param list<string> $parentInterfaces
     * @param list<string> $requiredExtends
     * @param list<string> $requiredImplements
     * @param list<string> $usedTraits
     * @param list<MethodDefinition> $methods
     * @param list<PropertyDefinition> $properties
     * @param list<PropertyDefinition> $magicProperties
     * @param list<ClassConstantDefinition> $constants
     * @param list<EnumCaseDefinition> $enumCases
     * @param list<TemplateDefinition> $templates
     * @param array<string, list<Type>> $extendedTypes
     * @param array<string, Type> $typeAliases
     * @param list<Type> $mixins
     * @param list<string>|null $permittedInheritors
     */
    public function __construct(
        public readonly string $name,
        public readonly ClassLikeKind $kind = ClassLikeKind::Class_,
        public readonly ?string $parentClass = null,
        public readonly array $parentInterfaces = [],
        public readonly array $requiredExtends = [],
        public readonly array $requiredImplements = [],
        public readonly array $usedTraits = [],
        public readonly ?Type $enumType = null,
        public readonly array $methods = [],
        public readonly array $properties = [],
        public readonly array $magicProperties = [],
        public readonly array $constants = [],
        public readonly array $enumCases = [],
        public readonly array $templates = [],
        public readonly array $extendedTypes = [],
        public readonly array $typeAliases = [],
        public readonly array $mixins = [],
        public readonly ?bool $sealedMethods = null,
        public readonly ?bool $sealedProperties = null,
        public readonly ?array $permittedInheritors = null,
        public readonly int $flags = 0,
    ) {
        DefinitionName::assertSymbol($name, 'A class-like definition name');
        if ($parentClass !== null) {
            DefinitionName::assertSymbol($parentClass, 'A parent class name');
        }
    }
}
