<?php

declare(strict_types=1);

/**
 * @param callable(int, int...): int $f
 */
function applyBA(callable $f): int
{
    return $f(1, 2, 3);
}

echo applyBA(static fn(int ...$ns): int => array_sum($ns));
