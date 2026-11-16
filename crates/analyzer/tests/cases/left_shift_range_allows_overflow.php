<?php

declare(strict_types=1);

/**
 * @param non-negative-int $value
 */
function shifted_unbounded(int $value): void
{
    if (($value << 1) < 0) {
        echo 'overflowed';
    }
}

function shifted_in_a_loop(): void
{
    $value = 1;
    for ($i = 0; $i < 70; $i++) {
        $value <<= 1;
    }

    if ($value < 2) {
        echo 'shifted out';
    }
}

/**
 * @param int<0, 3> $value
 * @param int<1, 2> $shift
 *
 * @return int<0, 12>
 */
function shifted_bounded(int $value, int $shift): int
{
    return $value << $shift;
}
