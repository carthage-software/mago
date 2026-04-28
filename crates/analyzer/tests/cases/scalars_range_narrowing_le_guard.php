<?php

declare(strict_types=1);

/** @param non-positive-int $n */
function np(int $n): int { return $n; }

function example(int $x): void {
    if ($x <= 0) {
        np($x);
    }
}
