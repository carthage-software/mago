<?php

/*
 * This file is part of the API Platform project.
 *
 * (c) Kévin Dunglas <dunglas@gmail.com>
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */

declare(strict_types=1);

namespace ApiPlatform\JsonSchema\Metadata\Property\Factory;

use ApiPlatform\JsonSchema\Schema;
use ApiPlatform\Metadata\ApiProperty;
use ApiPlatform\Metadata\Exception\PropertyNotFoundException;
use ApiPlatform\Metadata\Property\Factory\PropertyMetadataFactoryInterface;
use ApiPlatform\Metadata\ResourceClassResolverInterface;
use ApiPlatform\Metadata\Util\ResourceClassInfoTrait;
use Doctrine\Common\Collections\ArrayCollection;
use Ramsey\Uuid\UuidInterface;
use Symfony\Component\PropertyInfo\PropertyInfoExtractor;
use Symfony\Component\PropertyInfo\Type as LegacyType;
use Symfony\Component\TypeInfo\Type;
use Symfony\Component\TypeInfo\Type\BuiltinType;
use Symfony\Component\TypeInfo\Type\CollectionType;
use Symfony\Component\TypeInfo\Type\IntersectionType;
use Symfony\Component\TypeInfo\Type\ObjectType;
use Symfony\Component\TypeInfo\Type\UnionType;
use Symfony\Component\TypeInfo\Type\WrappingTypeInterface;
use Symfony\Component\TypeInfo\TypeIdentifier;
use Symfony\Component\Uid\Ulid;
use Symfony\Component\Uid\Uuid;

/**
 * Build ApiProperty::schema.
 */
final class SchemaPropertyMetadataFactory implements PropertyMetadataFactoryInterface
{
    use ResourceClassInfoTrait;

    public const JSON_SCHEMA_USER_DEFINED = 'user_defined_schema';

    public function __construct(
        ResourceClassResolverInterface $resourceClassResolver,
        private readonly ?PropertyMetadataFactoryInterface $decorated = null,
    ) {
        $this->resourceClassResolver = $resourceClassResolver;
    }

    public function create(string $resourceClass, string $property, array $options = []): ApiProperty
    {
        if (null === $this->decorated) {
            $propertyMetadata = new ApiProperty();
        } else {
            try {
                $propertyMetadata = $this->decorated->create($resourceClass, $property, $options);
            } catch (PropertyNotFoundException) {
                $propertyMetadata = new ApiProperty();
            }
        }

        $extraProperties = $propertyMetadata->getExtraProperties();
        // see AttributePropertyMetadataFactory
        if (true === ($extraProperties[self::JSON_SCHEMA_USER_DEFINED] ?? false)) {
            // schema seems to have been declared by the user: do not override nor complete user value
            return $propertyMetadata;
        }

        $link = (($options['schema_type'] ?? null) === Schema::TYPE_INPUT) ? $propertyMetadata->isWritableLink() : $propertyMetadata->isReadableLink();
        $propertySchema = $propertyMetadata->getSchema() ?? [];

        if (null !== $propertyMetadata->getUriTemplate() || (!\array_key_exists('readOnly', $propertySchema) && false === $propertyMetadata->isWritable() && !$propertyMetadata->isInitializable())) {
            $propertySchema['readOnly'] = true;
        }

        if (!\array_key_exists('writeOnly', $propertySchema) && false === $propertyMetadata->isReadable()) {
            $propertySchema['writeOnly'] = true;
        }

        if (!\array_key_exists('description', $propertySchema) && null !== ($description = $propertyMetadata->getDescription())) {
            $propertySchema['description'] = $description;
        }

        // see https://github.com/json-schema-org/json-schema-spec/pull/737
        if (!\array_key_exists('deprecated', $propertySchema) && null !== $propertyMetadata->getDeprecationReason()) {
            $propertySchema['deprecated'] = true;
        }

        // externalDocs is an OpenAPI specific extension, but JSON Schema allows additional keys, so we always add it
        // See https://json-schema.org/latest/json-schema-core.html#rfc.section.6.4
        if (!\array_key_exists('externalDocs', $propertySchema) && null !== ($iri = $propertyMetadata->getTypes()[0] ?? null)) {
            $propertySchema['externalDocs'] = ['url' => $iri];
        }

        if (!method_exists(PropertyInfoExtractor::class, 'getType')) {
            return $propertyMetadata->withSchema($this->getLegacyTypeSchema($propertyMetadata, $propertySchema, $resourceClass, $property, $link));
        }

        return $propertyMetadata->withSchema($this->getTypeSchema($propertyMetadata, $propertySchema, $link));
    }

    private function getTypeSchema(ApiProperty $propertyMetadata, array $propertySchema, ?bool $link): array
    {
        $type = $propertyMetadata->getNativeType();

        $className = null;
        $typeIsResourceClass = function (Type $type) use (&$className): bool {
            return $type instanceof ObjectType && $this->resourceClassResolver->isResourceClass($className = $type->getClassName());
        };
        $isResourceClass = $type?->isSatisfiedBy($typeIsResourceClass);

        if (null !== $propertyMetadata->getUriTemplate() || (!\array_key_exists('readOnly', $propertySchema) && false === $propertyMetadata->isWritable() && !$propertyMetadata->isInitializable()) && !$className) {
            $propertySchema['readOnly'] = true;
        }

        if (!\array_key_exists('default', $propertySchema) && null !== ($default = $propertyMetadata->getDefault()) && false === (\is_array($default) && empty($default)) && !$isResourceClass) {
            if ($default instanceof \BackedEnum) {
                $default = $default->value;
            }
            $propertySchema['default'] = $default;
        }

        if (!\array_key_exists('example', $propertySchema) && null !== ($example = $propertyMetadata->getExample()) && false === (\is_array($example) && empty($example))) {
            $propertySchema['example'] = $example;
        }

        $hasType = $this->getSchemaValue($propertySchema, 'type') || $this->getSchemaValue($propertyMetadata->getJsonSchemaContext() ?? [], 'type') || $this->getSchemaValue($propertyMetadata->getOpenapiContext() ?? [], 'type');
        $hasRef = $this->getSchemaValue($propertySchema, '$ref') || $this->getSchemaValue($propertyMetadata->getJsonSchemaContext() ?? [], '$ref') || $this->getSchemaValue($propertyMetadata->getOpenapiContext() ?? [], '$ref');

        // never override the following keys if at least one is already set or if there's a custom openapi context
        if ($hasType || $hasRef || !$type) {
            return $propertySchema;
        }

        if ($type instanceof CollectionType && null !== $propertyMetadata->getUriTemplate()) {
            $type = $type->getCollectionValueType();
        }

        return $propertySchema + $this->getJsonSchemaFromType($type, $link);
    }

    /**
     * Applies nullability rules to a generated JSON schema based on the original type's nullability.
     *
     * @param array<string, mixed> $schema     the base JSON schema generated for the non-null type
     * @param bool                 $isNullable whether the original type allows null
     *
     * @return array<string, mixed> the JSON schema with nullability applied
     */
    private function applyNullability(array $schema, bool $isNullable): array
    {
        if (!$isNullable) {
            return $schema;
        }

        if (isset($schema['type']) && 'null' === $schema['type'] && 1 === \count($schema)) {
            return $schema;
        }

        if (isset($schema['anyOf']) && \is_array($schema['anyOf'])) {
            $hasNull = false;
            foreach ($schema['anyOf'] as $anyOfSchema) {
                if (isset($anyOfSchema['type']) && 'null' === $anyOfSchema['type']) {
                    $hasNull = true;
                    break;
                }
            }
            if (!$hasNull) {
                $schema['anyOf'][] = ['type' => 'null'];
            }

            return $schema;
        }

        if (isset($schema['type'])) {
            $currentType = $schema['type'];
            $schema['type'] = \is_array($currentType) ? array_merge($currentType, ['null']) : [$currentType, 'null'];

            if (isset($schema['enum'])) {
                $schema['enum'][] = null;

                return $schema;
            }

            return $schema;
        }

        return ['anyOf' => [$schema, ['type' => 'null']]];
    }

    /**
     * Converts a TypeInfo Type into a JSON Schema definition array.
     *
     * @return array<string, mixed>
     */
    private function getJsonSchemaFromType(Type $type, ?bool $readableLink = null): array
    {
        $isNullable = $type->isNullable();

        if ($type instanceof UnionType) {
            $subTypes = array_filter($type->getTypes(), static fn ($t) => !($t instanceof BuiltinType && $t->isIdentifiedBy(TypeIdentifier::NULL)));

            foreach ($subTypes as $t) {
                $s = $this->getJsonSchemaFromType($t, $readableLink);
                // We can not find what type this is, let it be computed at runtime by the SchemaFactory
                if (($s['type'] ?? null) === Schema::UNKNOWN_TYPE) {
                    return $s;
                }
            }

            $schemas = array_map(fn ($t) => $this->getJsonSchemaFromType($t, $readableLink), $subTypes);

            if (0 === \count($schemas)) {
                $schema = [];
            } elseif (1 === \count($schemas)) {
                $schema = current($schemas);
            } else {
                $schema = ['anyOf' => array_values($schemas)];
            }

            return $this->applyNullability($schema, $isNullable);
        }

        if ($type instanceof IntersectionType) {
            $schemas = [];
            foreach ($type->getTypes() as $t) {
                while ($t instanceof WrappingTypeInterface) {
                    $t = $t->getWrappedType();
                }

                $subSchema = $this->getJsonSchemaFromType($t, $readableLink);
                if (!empty($subSchema)) {
                    $schemas[] = $subSchema;
                }
            }

            return $this->applyNullability(['allOf' => $schemas], $isNullable);
        }

        if ($type instanceof CollectionType) {
            $valueType = $type->getCollectionValueType();
            $valueSchema = $this->getJsonSchemaFromType($valueType, $readableLink);
            $keyType = $type->getCollectionKeyType();

            // Associative array (string keys)
            if ($keyType->isSatisfiedBy(static fn (Type $t) => $t instanceof BuiltinType && $t->isIdentifiedBy(TypeIdentifier::INT))) {
                $schema = [
                    'type' => 'array',
                    'items' => $valueSchema,
                ];
            } else { // List (int keys)
                $schema = [
                    'type' => 'object',
                    'additionalProperties' => $valueSchema,
                ];
            }

            return $this->applyNullability($schema, $isNullable);
        }

        if ($type instanceof ObjectType) {
            $schema = $this->getClassSchemaDefinition($type->getClassName(), $readableLink);

            return $this->applyNullability($schema, $isNullable);
        }

        if ($type instanceof BuiltinType) {
            $schema = match ($type->getTypeIdentifier()) {
                TypeIdentifier::INT => ['type' => 'integer'],
                TypeIdentifier::FLOAT => ['type' => 'number'],
                TypeIdentifier::BOOL => ['type' => 'boolean'],
                TypeIdentifier::TRUE => ['type' => 'boolean', 'const' => true],
                TypeIdentifier::FALSE => ['type' => 'boolean', 'const' => false],
                TypeIdentifier::STRING => ['type' => 'string'],
                TypeIdentifier::ARRAY => ['type' => 'array', 'items' => []],
                TypeIdentifier::ITERABLE => ['type' => 'array', 'items' => []],
                TypeIdentifier::OBJECT => ['type' => 'object'],
                TypeIdentifier::RESOURCE => ['type' => 'string'],
                TypeIdentifier::CALLABLE => ['type' => 'string'],
                TypeIdentifier::MIXED => ['type' => 'string'],
                default => ['type' => 'null'],
            };

            return $this->applyNullability($schema, $isNullable);
        }

        return ['type' => Schema::UNKNOWN_TYPE];
    }

    /**
     * Gets the JSON Schema definition for a class.
     */
    private function getClassSchemaDefinition(?string $className, ?bool $readableLink): array
    {
        if (null === $className) {
            return ['type' => 'string'];
        }

        if (is_a($className, \DateTimeInterface::class, true)) {
            return ['type' => 'string', 'format' => 'date-time'];
        }

        if (is_a($className, \DateInterval::class, true)) {
            return ['type' => 'string', 'format' => 'duration'];
        }

        if (is_a($className, UuidInterface::class, true) || is_a($className, Uuid::class, true)) {
            return ['type' => 'string', 'format' => 'uuid'];
        }

        if (is_a($className, Ulid::class, true)) {
            return ['type' => 'string', 'format' => 'ulid'];
        }

        if (is_a($className, \SplFileInfo::class, true)) {
            return ['type' => 'string', 'format' => 'binary'];
        }

        if (is_a($className, \BcMath\Number::class, true)) {
            return ['type' => 'string', 'format' => 'string'];
        }

        $isResourceClass = $this->isResourceClass($className);
        if (!$isResourceClass && is_a($className, \BackedEnum::class, true)) {
            $enumCases = array_map(static fn (\BackedEnum $enum): string|int => $enum->value, $className::cases());
            $type = \is_string($enumCases[0] ?? '') ? 'string' : 'integer';

            return ['type' => $type, 'enum' => $enumCases];
        }

        if (false === $readableLink && $isResourceClass) {
            return [
                'type' => 'string',
                'format' => 'iri-reference',
                'example' => 'https://example.com/',
            ];
        }

        return ['type' => Schema::UNKNOWN_TYPE];
    }

    private function getLegacyTypeSchema(ApiProperty $propertyMetadata, array $propertySchema, string $resourceClass, string $property, ?bool $link): array
    {
        $types = $propertyMetadata->getBuiltinTypes() ?? [];
        $className = ($types[0] ?? null)?->getClassName() ?? null;

        if (null !== $propertyMetadata->getUriTemplate() || (!\array_key_exists('readOnly', $propertySchema) && false === $propertyMetadata->isWritable() && !$propertyMetadata->isInitializable()) && !$className) {
            $propertySchema['readOnly'] = true;
        }

        if (!\array_key_exists('default', $propertySchema) && !empty($default = $propertyMetadata->getDefault()) && (!$className || !$this->isResourceClass($className))) {
            if ($default instanceof \BackedEnum) {
                $default = $default->value;
            }
            $propertySchema['default'] = $default;
        }

        if (!\array_key_exists('example', $propertySchema) && !empty($example = $propertyMetadata->getExample())) {
            $propertySchema['example'] = $example;
        }

        // never override the following keys if at least one is already set or if there's a custom openapi context
        if (
            [] === $types
            || ($propertySchema['type'] ?? $propertySchema['$ref'] ?? $propertySchema['anyOf'] ?? $propertySchema['allOf'] ?? $propertySchema['oneOf'] ?? false)
            || \array_key_exists('type', $propertyMetadata->getOpenapiContext() ?? [])
        ) {
            return $propertySchema;
        }

        if ($propertyMetadata->getUriTemplate()) {
            return $propertySchema + [
                'type' => 'string',
                'format' => 'iri-reference',
                'example' => 'https://example.com/',
            ];
        }

        $valueSchema = [];
        foreach ($types as $type) {
            // Temp fix for https://github.com/symfony/symfony/pull/52699
            if (ArrayCollection::class === $type->getClassName()) {
                $type = new LegacyType($type->getBuiltinType(), $type->isNullable(), $type->getClassName(), true, $type->getCollectionKeyTypes(), $type->getCollectionValueTypes());
            }

            if ($isCollection = $type->isCollection()) {
                $keyType = $type->getCollectionKeyTypes()[0] ?? null;
                $valueType = $type->getCollectionValueTypes()[0] ?? null;
            } else {
                $keyType = null;
                $valueType = $type;
            }

            if (null === $valueType) {
                $builtinType = 'string';
                $className = null;
            } else {
                $builtinType = $valueType->getBuiltinType();
                $className = $valueType->getClassName();
            }

            if ($isCollection && null !== $propertyMetadata->getUriTemplate()) {
                $keyType = null;
                $isCollection = false;
            }

            $propertyType = $this->getLegacyType(new LegacyType($builtinType, $type->isNullable(), $className, $isCollection, $keyType, $valueType), $link);
            if (!\in_array($propertyType, $valueSchema, true)) {
                $valueSchema[] = $propertyType;
            }
        }

        if (1 === \count($valueSchema)) {
            return $propertySchema + $valueSchema[0];
        }

        // multiple builtInTypes detected: determine oneOf/allOf if union vs intersect types
        try {
            $reflectionClass = new \ReflectionClass($resourceClass);
            $reflectionProperty = $reflectionClass->getProperty($property);
            $composition = $reflectionProperty->getType() instanceof \ReflectionUnionType ? 'oneOf' : 'allOf';
        } catch (\ReflectionException) {
            // cannot detect types
            $composition = 'anyOf';
        }

        return $propertySchema + [$composition => $valueSchema];
    }

    private function getLegacyType(LegacyType $type, ?bool $readableLink = null): array
    {
        if (!$type->isCollection()) {
            return $this->addNullabilityToTypeDefinition($this->legacyTypeToArray($type, $readableLink), $type);
        }

        $keyType = $type->getCollectionKeyTypes()[0] ?? null;
        $subType = ($type->getCollectionValueTypes()[0] ?? null) ?? new LegacyType($type->getBuiltinType(), false, $type->getClassName(), false);

        if (null !== $keyType && LegacyType::BUILTIN_TYPE_STRING === $keyType->getBuiltinType()) {
            return $this->addNullabilityToTypeDefinition([
                'type' => 'object',
                'additionalProperties' => $this->getLegacyType($subType, $readableLink),
            ], $type);
        }

        return $this->addNullabilityToTypeDefinition([
            'type' => 'array',
            'items' => $this->getLegacyType($subType, $readableLink),
        ], $type);
    }

    private function legacyTypeToArray(LegacyType $type, ?bool $readableLink = null): array
    {
        return match ($type->getBuiltinType()) {
            LegacyType::BUILTIN_TYPE_INT => ['type' => 'integer'],
            LegacyType::BUILTIN_TYPE_FLOAT => ['type' => 'number'],
            LegacyType::BUILTIN_TYPE_BOOL => ['type' => 'boolean'],
            LegacyType::BUILTIN_TYPE_OBJECT => $this->getLegacyClassType($type->getClassName(), $type->isNullable(), $readableLink),
            default => ['type' => 'string'],
        };
    }

    /**
     * Gets the JSON Schema document which specifies the data type corresponding to the given PHP class, and recursively adds needed new schema to the current schema if provided.
     *
     * Note: if the class is not part of exceptions listed above, any class is considered as a resource.
     *
     * @throws PropertyNotFoundException
     *
     * @return array<string, mixed>
     */
    private function getLegacyClassType(?string $className, bool $nullable, ?bool $readableLink): array
    {
        if (null === $className) {
            return ['type' => 'string'];
        }

        if (is_a($className, \DateTimeInterface::class, true)) {
            return [
                'type' => 'string',
                'format' => 'date-time',
            ];
        }

        if (is_a($className, \DateInterval::class, true)) {
            return [
                'type' => 'string',
                'format' => 'duration',
            ];
        }

        if (is_a($className, UuidInterface::class, true) || is_a($className, Uuid::class, true)) {
            return [
                'type' => 'string',
                'format' => 'uuid',
            ];
        }

        if (is_a($className, Ulid::class, true)) {
            return [
                'type' => 'string',
                'format' => 'ulid',
            ];
        }

        if (is_a($className, \SplFileInfo::class, true)) {
            return [
                'type' => 'string',
                'format' => 'binary',
            ];
        }

        if (is_a($className, \BcMath\Number::class, true)) {
            return [
                'type' => 'string',
                'format' => 'string',
            ];
        }

        $isResourceClass = $this->isResourceClass($className);
        if (!$isResourceClass && is_a($className, \BackedEnum::class, true)) {
            $enumCases = array_map(static fn (\BackedEnum $enum): string|int => $enum->value, $className::cases());

            $type = \is_string($enumCases[0] ?? '') ? 'string' : 'integer';

            if ($nullable) {
                $enumCases[] = null;
            }

            return [
                'type' => $type,
                'enum' => $enumCases,
            ];
        }

        if (false === $readableLink && $isResourceClass) {
            return [
                'type' => 'string',
                'format' => 'iri-reference',
                'example' => 'https://example.com/',
            ];
        }

        // When this is set, we compute the schema at SchemaFactory::buildPropertySchema as it
        // will end up being a $ref to another class schema, we don't have enough informations here
        return ['type' => Schema::UNKNOWN_TYPE];
    }

    /**
     * @param array<string, mixed> $jsonSchema
     *
     * @return array<string, mixed>
     */
    private function addNullabilityToTypeDefinition(array $jsonSchema, LegacyType $type): array
    {
        if (!$type->isNullable()) {
            return $jsonSchema;
        }

        if (\array_key_exists('$ref', $jsonSchema)) {
            return ['anyOf' => [$jsonSchema, ['type' => 'null']]];
        }

        return [...$jsonSchema, ...[
            'type' => \is_array($jsonSchema['type'])
                ? array_merge($jsonSchema['type'], ['null'])
                : [$jsonSchema['type'], 'null'],
        ]];
    }

    private function getSchemaValue(array $schema, string $key): array|string|null
    {
        if (isset($schema['items'])) {
            $schema = $schema['items'];
        }

        return $schema[$key] ?? $schema['allOf'][0][$key] ?? $schema['anyOf'][0][$key] ?? $schema['oneOf'][0][$key] ?? null;
    }
}
