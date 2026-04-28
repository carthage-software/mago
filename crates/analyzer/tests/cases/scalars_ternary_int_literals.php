<?php

declare(strict_types=1);

/** @param 1|2 $n */
function one_or_two(int $n): int { return $n; }

function example(bool $b): int {
    $x = $b ? 1 : 2;
    return one_or_two($x);
}
