<?php

declare(strict_types=1);

/** @param 1|2|3 $n */
function r(int $n): int { return $n; }

function example(int $n): int {
    return match ($n) {
        1, 2, 3 => r($n),
        default => 1,
    };
}
