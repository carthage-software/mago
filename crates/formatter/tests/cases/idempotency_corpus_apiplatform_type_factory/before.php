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

namespace ApiPlatform\OpenApi\Factory;

use Ramsey\Uuid\UuidInterface;
use Symfony\Component\PropertyInfo\Type as LegacyType;
use Symfony\Component\TypeInfo\Type as NativeType;
use Symfony\Component\TypeInfo\Type\CollectionType;
use Symfony\Component\TypeInfo\Type\ObjectType;
use Symfony\Component\TypeInfo\TypeIdentifier;
use Symfony\Component\Uid\Ulid;
use Symfony\Component\Uid\Uuid;

/**
 * @internal
 */
trait TypeFactoryTrait
{
    /**
     * @return array<string, mixed>
     */
    private function getType(LegacyType|NativeType $type): array
    {
        if ($type instanceof NativeType) {
            return $this->getNativeType($type);
        }

        if ($type->isCollection()) {
            $keyType = $type->getCollectionKeyTypes()[0] ?? null;
            $subType = ($type->getCollectionValueTypes()[0] ?? null) ?? new LegacyType($type->getBuiltinType(), false, $type->getClassName(), false);

            if (null !== $keyType && LegacyType::BUILTIN_TYPE_STRING === $keyType->getBuiltinType()) {
                return $this->addNullabilityToTypeDefinition([
                    'type' => 'object',
                    'additionalProperties' => $this->getType($subType),
                ], $type);
            }

            return $this->addNullabilityToTypeDefinition([
                'type' => 'array',
                'items' => $this->getType($subType),
            ], $type);
        }

        return $this->addNullabilityToTypeDefinition($this->makeLegacyBasicType($type), $type);
    }

    /**
     * @return array<string, mixed>
     */
    private function getNativeType(NativeType $type): array
    {
        if ($type instanceof CollectionType) {
            $keyType = $type->getCollectionKeyType();
            $subType = $type->getCollectionValueType();

            if ($keyType->isIdentifiedBy(TypeIdentifier::STRING)) {
                return $this->addNullabilityToTypeDefinition([
                    'type' => 'object',
                    'additionalProperties' => $this->getNativeType($subType),
                ], $type);
            }

            return $this->addNullabilityToTypeDefinition([
                'type' => 'array',
                'items' => $this->getNativeType($subType),
            ], $type);
        }

        return $this->addNullabilityToTypeDefinition($this->makeBasicType($type), $type);
    }

    /**
     * @return array<string, mixed>
     */
    private function makeLegacyBasicType(LegacyType $type): array
    {
        return match ($type->getBuiltinType()) {
            LegacyType::BUILTIN_TYPE_INT => ['type' => 'integer'],
            LegacyType::BUILTIN_TYPE_FLOAT => ['type' => 'number'],
            LegacyType::BUILTIN_TYPE_BOOL => ['type' => 'boolean'],
            LegacyType::BUILTIN_TYPE_OBJECT => $this->getClassType($type->getClassName(), $type->isNullable()),
            default => ['type' => 'string'],
        };
    }

    /**
     * @return array<string, mixed>
     */
    private function makeBasicType(NativeType $type): array
    {
        if ($type->isIdentifiedBy(TypeIdentifier::INT)) {
            return ['type' => 'integer'];
        }
        if ($type->isIdentifiedBy(TypeIdentifier::FLOAT)) {
            return ['type' => 'number'];
        }
        if ($type->isIdentifiedBy(TypeIdentifier::BOOL)) {
            return ['type' => 'boolean'];
        }
        if ($type instanceof ObjectType) {
            return $this->getClassType($type->getClassName(), $type->isNullable());
        }

        // Default for other built-in types like string, resource, mixed, etc.
        return ['type' => 'string'];
    }

    /**
     * Gets the JSON Schema document which specifies the data type corresponding to the given PHP class, and recursively adds needed new schema to the current schema if provided.
     *
     * @return array<string, mixed>
     */
    private function getClassType(?string $className, bool $nullable): array
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

        if (is_a($className, \BackedEnum::class, true)) {
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

        return ['type' => 'string'];
    }

    /**
     * @param array<string, mixed> $jsonSchema
     *
     * @return array<string, mixed>
     */
    private function addNullabilityToTypeDefinition(array $jsonSchema, LegacyType|NativeType $type): array
    {
        if (!$type->isNullable()) {
            return $jsonSchema;
        }

        $typeDefinition = ['anyOf' => [$jsonSchema]];
        $typeDefinition['anyOf'][] = ['type' => 'null'];

        return $typeDefinition;
    }
}
