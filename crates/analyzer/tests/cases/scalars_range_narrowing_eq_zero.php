<?php

declare(strict_types=1);

/** @param int<0, 0> $n */
function zero(int $n): int { return $n; }

function example(int $x): void {
    if ($x === 0) {
        zero($x);
    }
}
