<?php

declare(strict_types=1);

function callables_factorial(int $n): int
{
    if ($n <= 1) {
        return 1;
    }
    return $n * callables_factorial($n - 1);
}

echo callables_factorial(5);
