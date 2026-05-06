<?php

declare(strict_types=1);

function callables_sum(int ...$nums): int
{
    $total = 0;
    foreach ($nums as $n) {
        $total += $n;
    }
    return $total;
}

echo callables_sum();
echo callables_sum(1);
echo callables_sum(1, 2, 3, 4, 5);
