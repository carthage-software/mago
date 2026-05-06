<?php

declare(strict_types=1);

function callables_sum_ints(int ...$nums): int
{
    $total = 0;
    foreach ($nums as $n) {
        $total += $n;
    }
    return $total;
}

callables_sum_ints(1, 2, 'three');
