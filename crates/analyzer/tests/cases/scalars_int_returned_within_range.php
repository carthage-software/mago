<?php

declare(strict_types=1);

/**
 * @return int<0, 100>
 */
function clamped(int $x): int {
    if ($x < 0) {
        return 0;
    }
    if ($x > 100) {
        return 100;
    }
    return $x;
}

clamped(50);
