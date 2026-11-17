<?php

declare(strict_types=1);

function take_two_ints(int $first, int $second): void
{
}

function unpack_object(stdClass $value): void
{
    /** @mago-expect analysis:invalid-argument */
    take_two_ints(...$value);
}

function unpack_int(int $value): void
{
    /** @mago-expect analysis:invalid-argument */
    take_two_ints(...$value);
}

/**
 * @param list<string> $values
 */
function unpack_string_list(array $values): void
{
    /** @mago-expect analysis:invalid-argument(2) */
    take_two_ints(...$values);
}

/**
 * @param list<int> $values
 */
function unpack_int_list(array $values): void
{
    take_two_ints(...$values);
}

/**
 * @param list<int|string> $values
 */
function unpack_mixed_list(array $values): void
{
    take_two_ints(...$values);
}
