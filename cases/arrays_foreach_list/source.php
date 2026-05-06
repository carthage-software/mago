<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function sum_via_foreach(array $xs): int
{
    $total = 0;
    foreach ($xs as $x) {
        $total += $x;
    }
    return $total;
}
