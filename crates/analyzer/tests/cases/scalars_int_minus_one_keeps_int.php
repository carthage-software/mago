<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

/** @param int<5, 10> $x */
function example(int $x): void {
    $y = $x - 1;
    takesInt($y);
}
