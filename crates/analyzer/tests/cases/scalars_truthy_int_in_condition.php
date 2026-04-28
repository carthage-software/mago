<?php

declare(strict_types=1);

function takesNonZero(int $n): int { return $n; }

function example(int $x): int {
    if ($x) {
        return takesNonZero($x);
    }
    return 0;
}
