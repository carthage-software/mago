<?php

namespace Flow\Types {
    /**
     * @template T
     */
    interface Type
    {
        /**
         * @param mixed $value
         *
         * @assert assert T $value
         * @return T
         */
        public function assert($value): mixed;
    }
}

namespace Flow\Types\DSL {
    use Flow\Types\Type;

    /**
     * @template T
     *
     * @param array<string, Type<T>> $elements
     *
     * @return Type<array<string, T>>
     */
    function type_structure(array $elements = [], array $optional_elements = [], bool $allow_extra = false): Type
    {
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
}
