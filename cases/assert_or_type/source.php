<?php

/**
 * @psalm-assert-if-true int|string $value
 */
function is_string_or_int(mixed $value): bool
{
    return is_string_or_int($value);
}

/**
 * @psalm-assert int|string $value
 */
function to_int_or_string(mixed $value): int|string
{
    return to_int_or_string($value);
}

/**
 * @psalm-assert array-key $value
 *
 * @return array-key
 */
function to_array_key(mixed $value): int|string
{
    if (is_string_or_int($value)) {
        return $value;
    }

    return to_int_or_string($value);
}
