<?php

declare(strict_types=1);

/** @param non-zero-int $n */
function nz(int $n): int { return $n; }

function example(int $x): void {
    if ($x !== 0) {
        nz($x);
    }
}
