<?php

declare(strict_types=1);

function literal_overflow_is_not_capped(): string
{
    $x = 9223372036854775807 + 1;
    if ($x === 9223372036854775807) {
        return 'capped';
    }

    return 'overflowed';
}

/** @param int<4611686018427387904, 4611686018427387905> $h */
function range_overflow_is_not_capped(int $h): string
{
    $p = $h * 2;
    if ($p === 9223372036854775807) {
        return 'capped';
    }

    return 'overflowed';
}

/**
 * @param positive-int $p
 * @param negative-int $n
 *
 * @return negative-int
 */
function positive_times_negative(int $p, int $n): int
{
    return $p * $n;
}

/**
 * @param negative-int $a
 * @param negative-int $b
 *
 * @return positive-int
 */
function negative_times_negative(int $a, int $b): int
{
    return $a * $b;
}

/**
 * @param positive-int $p
 * @param int<2, 5> $r
 *
 * @return int<2, max>
 */
function positive_times_range(int $p, int $r): int
{
    return $p * $r;
}

/**
 * @param int<min, -1> $n
 * @param int<0, 5> $r
 *
 * @return non-positive-int
 */
function negative_times_range(int $n, int $r): int
{
    return $n * $r;
}
