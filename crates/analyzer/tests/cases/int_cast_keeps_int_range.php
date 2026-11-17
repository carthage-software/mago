<?php

declare(strict_types=1);

/**
 * @param int<0, 20> $value
 */
function take_small_int(int $value): void
{
}

/**
 * @param positive-int $value
 */
function take_positive_int(int $value): void
{
}

/**
 * @param false|int<10, 20> $a
 * @param positive-int $b
 * @param bool|int<10, 20> $c
 */
function cast_ranges(false|int $a, int $b, bool|int $c): void
{
    take_small_int((int) $a);
    take_positive_int((int) $b);
    take_small_int((int) $c);
}
