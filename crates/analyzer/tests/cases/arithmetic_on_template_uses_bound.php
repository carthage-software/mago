<?php

declare(strict_types=1);

/**
 * @template T of int|float
 *
 * @param T $n
 */
function arithmetic_template_remainder(int|float $n): int
{
    return $n % 2;
}

/**
 * @template T of int
 *
 * @param T $n
 */
function arithmetic_template_next(int $n): int
{
    $n++;

    return $n;
}

/**
 * @template T of int
 *
 * @param T $n
 */
function arithmetic_template_previous(int $n): int
{
    --$n;

    return $n;
}

/**
 * @template T of int
 *
 * @param T $n
 *
 * @return T
 */
function arithmetic_template_increment(int $n): int
{
    // @mago-expect analysis:invalid-return-statement
    return $n + 1;
}
