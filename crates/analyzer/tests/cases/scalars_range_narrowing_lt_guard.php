<?php

declare(strict_types=1);

/** @param negative-int $n */
function neg(int $n): int { return $n; }

function example(int $x): void {
    if ($x < 0) {
        neg($x);
    }
}
