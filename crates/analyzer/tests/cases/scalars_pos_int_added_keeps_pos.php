<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

/**
 * @param positive-int $a
 * @param positive-int $b
 */
function example(int $a, int $b): int {
    return pos($a + $b);
}
