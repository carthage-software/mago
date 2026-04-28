<?php

declare(strict_types=1);

function callables_var_with_first(string $first, int ...$nums): int
{
    $sum = 0;
    foreach ($nums as $n) {
        $sum += $n;
    }
    return strlen($first) + $sum;
}

echo callables_var_with_first(first: 'hi', nums: 1);
