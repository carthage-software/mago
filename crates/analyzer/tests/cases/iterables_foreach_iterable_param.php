<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

/**
 * @param iterable<int> $it
 */
function sum_iter(iterable $it): int
{
    $total = 0;
    foreach ($it as $v) {
        take_int($v);
        $total += $v;
    }

    return $total;
}
