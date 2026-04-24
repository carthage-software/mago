<?php

declare(strict_types=1);

namespace Acme;

function boundary_check(float $v): bool
{
    $n = (int) $v;

    return $n === PHP_INT_MAX || $n === PHP_INT_MIN;
}

echo boundary_check((float) PHP_INT_MIN) ? "min\n" : "not min\n";
