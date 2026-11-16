<?php

declare(strict_types=1);

/**
 * @param positive-int $value
 */
function take_positive(int $value): void
{
}

/**
 * @template T of int
 *
 * @param T $value
 *
 * @return T
 */
function keep_positive(int $value): int
{
    if ($value > 0) {
        take_positive($value);
    }

    if ($value <= 0) {
        return $value;
    }

    take_positive($value);

    return $value;
}

/**
 * @template T of int<0, 3>
 *
 * @param T $value
 */
function compare_outside_bound(int $value): void
{
    if ($value > 10) { // @mago-expect analysis:impossible-condition,redundant-comparison
        echo 'never';
    }

    if ($value >= 0) { // @mago-expect analysis:redundant-condition,redundant-comparison
        echo 'always';
    }
}
