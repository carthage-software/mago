<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float { return $x; }

function example(int $a, int $b): int|float {
    if ($b === 0) {
        return 0;
    }
    return $a / $b;
}

takesIntFloat(example(7, 3));
