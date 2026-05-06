<?php

declare(strict_types=1);

/**
 * @template T of int
 * @param T $a
 * @return list<int>
 */
function enumerate(int $a, int $b): array
{
    return range($a, $b);
}

/**
 * @template T of 1|2|3
 * @param T $a
 * @return list<int>
 */
function enumerate_literal_ints(int $a, int $b): array
{
    // T is constrained to 1|2|3 which are all ints, so this should return list<int>
    return range($a, $b);
}

/**
 * @template T of 1.5|2.5|3.5
 * @param T $a
 * @return list<float>
 */
function enumerate_literal_floats(float $a, float $b): array
{
    // T is constrained to 1.5|2.5|3.5 which are all floats, so this should return list<float>
    return range($a, $b);
}

/**
 * @template T of int|float
 * @param T $a
 * @return list<int|float>
 */
function enumerate_mixed(int|float $a, int|float $b): array
{
    // T is constrained to int|float which is mixed, so this should return list<int|float>
    return range($a, $b);
}
