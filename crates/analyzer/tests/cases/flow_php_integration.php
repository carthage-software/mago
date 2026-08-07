<?php

namespace Flow\Types {
    /**
     * @template-covariant T
     */
    interface Type
    {
        /**
         * @param mixed $value
         *
         * @assert T $value
         * @return T
         */
        public function assert($value): mixed;
    }
}

namespace Flow\Types\Type\Logical {
    use Flow\Types\Type;

    /**
     * @template-covariant T of array<array-key, mixed>
     *
     * @implements Type<T>
     */
    final class StructureType implements Type
    {
        /**
         * @param array<array-key, Type<value-of<T>>> $elements
         * @param array<array-key, Type<value-of<T>>> $optionalElements
         */
        public function __construct(
            private array $elements = [],
            private array $optionalElements = [],
            private bool $allowExtra = false,
        ) {}

        public function allowsExtra(): bool
        {
            return $this->allowExtra;
        }

        /**
         * @param mixed $value
         *
         * @return T
         */
        public function assert($value): mixed
        {
            /** @var T $value */
            return $value;
        }

        /**
         * @return array<array-key, Type<mixed>>
         */
        public function elements(): array
        {
            return $this->elements;
        }

        /**
         * @return array<array-key, Type<mixed>>
         */
        public function optionalElements(): array
        {
            return $this->optionalElements;
        }
    }
}

namespace Flow\Types\DSL {
    use Flow\Types\Type;
    use Flow\Types\Type\Logical\StructureType;

    /**
     * @template T
     *
     * @param array<string, Type<T>> $elements
     *
     * @return StructureType<array<string, T>>
     */
    function type_structure(
        array $elements = [],
        array $optional_elements = [],
        bool $allow_extra = false,
    ): StructureType {
        return type_structure($elements, $optional_elements, $allow_extra);
    }

    /**
     * @return Type<string>
     */
    function type_string(): Type
    {
        return type_string();
    }

    /**
     * @return Type<int>
     */
    function type_integer(): Type
    {
        return type_integer();
    }
}

namespace {
    use Flow\Types\Type\Logical\StructureType;

    function get_mixed(): mixed
    {
        return 1;
    }

    function i_take_string(string $value): void
    {
        echo "Received string: $value\n";
    }

    function i_take_int(int $value): void
    {
        echo "Received int: $value\n";
    }

    function i_take_bool(bool $value): void
    {
        echo "Received bool: $value\n";
    }

    /**
     * The structural accessors consumers introspect with are declared on the concrete class, not on `Type`,
     * so the provider has to return `StructureType` for this parameter to be satisfiable from the DSL.
     *
     * @param StructureType<array<array-key, mixed>> $type
     */
    function i_take_structure_type(StructureType $type): void
    {
        i_take_int(count($type->elements()));
        i_take_int(count($type->optionalElements()));
        i_take_bool($type->allowsExtra());
    }

    /**
     * @param array{required_field: string, ...} $value
     * @return array{required_field: string, ...}
     */
    function i_take_flexible_array(array $value): array
    {
        return $value;
    }

    $array_type = Flow\Types\DSL\type_structure([
        'name' => Flow\Types\DSL\type_string(),
        'age' => Flow\Types\DSL\type_integer(),
        'address' => Flow\Types\DSL\type_structure([
            'street' => Flow\Types\DSL\type_string(),
            'city' => Flow\Types\DSL\type_string(),
        ], [
            'country' => Flow\Types\DSL\type_string(),
        ]),
    ]);

    // Test structure with allow_extra (third parameter)
    $flexible_type = Flow\Types\DSL\type_structure(['required_field' => Flow\Types\DSL\type_string()], [], true);

    $array = $array_type->assert(get_mixed());

    i_take_string($array['name']);
    i_take_int($array['age']);
    i_take_string($array['address']['street']);
    i_take_string($array['address']['city']);

    i_take_string($array['address']['country'] ?? 'DefaultCountry');

    if (isset($array['address']['country'])) {
        i_take_string($array['address']['country']);
    }

    $flexible = $flexible_type->assert(get_mixed());

    i_take_flexible_array($flexible);
    i_take_string($flexible['required_field']);

    // The inferred shape must remain assignable to the concrete class it is built from.
    i_take_structure_type($array_type);
    i_take_structure_type($flexible_type);
}
