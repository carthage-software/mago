<?php

declare(strict_types=1);

/**
 * @param 4|5 $m
 * @param positive-int $i
 */
function rounding_check(int $m, int $i): void
{
    if ($i >= $m || ($m % $i) !== 0) {
        echo "rejected: {$m} % {$i} = ", $m % $i, "\n";
    }
}
