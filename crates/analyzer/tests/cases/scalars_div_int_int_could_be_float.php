<?php

declare(strict_types=1);

function takesIntOrFloat(int|float $x): int|float { return $x; }

function divide(int $a, int $b): int|float {
    if ($b === 0) {
        return 0;
    }
    return $a / $b;
}

takesIntOrFloat(divide(10, 3));
