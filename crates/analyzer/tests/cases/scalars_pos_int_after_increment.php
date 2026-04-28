<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

/** @param non-negative-int $x */
function example(int $x): void {
    $y = $x + 1;  // non-negative + 1 -> at least 1
    pos($y);
}
