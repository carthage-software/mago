<?php

trait TypeFactoryTrait
{
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
        }

        return [];
    }
}
