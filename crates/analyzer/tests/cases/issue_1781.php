<?php

declare(strict_types=1);

/** @assert int $value */
function square_if_integer(mixed $value): int
{
    if (is_int($value)) {
        return $value ** 2;
    }

    throw new Exception('Value is not an integer');
}

function test(mixed $value): int
{
    $square = square_if_integer($value);

    return square_if_integer($square);
}

/**
 * @phpstan-assert string $s
 *
 * @pure
 */
function assert_string_transformed(mixed $s): string
{
    if (!is_string($s)) {
        throw new Exception('Value is not a string');
    }

    return $s . ' as string ';
}

function consume_assert_string_transformed(): string
{
    $a = 'a';
    $b = assert_string_transformed($a);

    return $b;
}
