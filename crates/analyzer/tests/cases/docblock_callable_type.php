<?php

declare(strict_types=1);

/**
 * @param callable(int): string $f
 */
function applyAY(callable $f): string
{
    return $f(7);
}

echo applyAY(static fn(int $n): string => (string) $n);
