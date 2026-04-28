<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

function example(int $x): void {
    if ($x > 0) {
        pos($x);
    }
}
