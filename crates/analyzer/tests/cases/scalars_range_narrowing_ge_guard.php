<?php

declare(strict_types=1);

/** @param non-negative-int $n */
function nn(int $n): int { return $n; }

function example(int $x): void {
    if ($x >= 0) {
        nn($x);
    }
}
