<?php

declare(strict_types=1);

/** @param int<1, 99> $n */
function r(int $n): int { return $n; }

function example(int $x): void {
    if ($x > 0 && $x < 100) {
        r($x);
    }
}
