<?php

declare(strict_types=1);

/** @param list<int> $a */
function product_of_ints(array $a): int
{
    return array_product($a);
}

/** @param list<float> $a */
function product_of_floats(array $a): float
{
    return array_product($a);
}

/**
 * @param array<bool> $a
 *
 * @return 0|1
 */
function all_true(array $a): int
{
    return array_product($a);
}

/**
 * @param list<true> $a
 *
 * @return 1
 */
function product_of_trues(array $a): int
{
    return array_product($a);
}

/** @return 1 */
function product_of_nothing(): int
{
    return array_product([]);
}
