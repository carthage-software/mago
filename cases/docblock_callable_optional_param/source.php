<?php

declare(strict_types=1);

/**
 * @param callable(int, string=): string $f
 */
function applyAZ(callable $f): string
{
    return $f(7);
}

echo applyAZ(static fn(int $n, string $s = ''): string => $s . $n);
