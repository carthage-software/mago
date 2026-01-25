<?php

function to_string_or_int(mixed $value): string|int
{
    return to_string_or_int($value);
}

function is_string_or_int(mixed $value): bool
{
    return is_string_or_int($value);
}

/**
 * @template T
 */
interface TypeInterface
{
    /**
     * @psalm-assert-if-true T $value
     */
    public function matches(mixed $value): bool;

    /**
     * @return T
     */
    public function coerce(mixed $value): mixed;

    /**
     * @return T
     */
    public function assert(mixed $value): mixed;
}

/**
 * @template T of BackedEnum
 *
 * @implements TypeInterface<value-of<T>>
 */
class EnumValueType implements TypeInterface
{
    /**
     * @param enum-string<T> $_enum
     */
    public function __construct(
        private string $_enum,
    ) {}

    /**
     * @psalm-assert-if-true =value-of<T> $value
     */
    public function matches(mixed $value): bool
    {
        return is_string_or_int($value);
    }

    /**
     * @return value-of<T>
     *
     * @throws InvalidArgumentException
     */
    public function coerce(mixed $value): string|int
    {
        $case = to_string_or_int($value);
        if ($this->matches($case)) {
            return $case;
        }

        throw new InvalidArgumentException('Invalid case');
    }

    /**
     * @return value-of<T>
     *
     * @throws InvalidArgumentException
     */
    public function assert(mixed $value): string|int
    {
        if ($this->matches($value)) {
            return $value;
        }

        throw new InvalidArgumentException('Invalid case');
    }
}

/**
 * @template T of BackedEnum
 *
 * @param enum-string<T> $enum
 *
 * @return TypeInterface<value-of<T>>
 */
function get_enum_value_type(string $enum): TypeInterface
{
    return new EnumValueType($enum);
}
