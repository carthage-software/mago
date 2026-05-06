<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: int, c: int} $shape
 */
function sum_shape_values(array $shape): int
{
    $total = 0;
    foreach ($shape as $v) {
        $total += $v;
    }
    return $total;
}
