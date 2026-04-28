<?php

declare(strict_types=1);

/** @param non-negative-int $n */
function nn(int $n): int { return $n; }

function clampToZero(int $x): int {
    if ($x < 0) {
        $x = 0;
    }
    return nn($x);
}

clampToZero(-5);
