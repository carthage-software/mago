<?php

declare(strict_types=1);

/** @param int<0, 100> $n */
function wide(int $n): int { return $n; }

/** @param int<10, 50> $x */
function caller(int $x): int {
    return wide($x);
}
