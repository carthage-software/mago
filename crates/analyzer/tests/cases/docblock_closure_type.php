<?php

declare(strict_types=1);

/**
 * @param Closure(int): string $f
 */
function applyAX(Closure $f): string
{
    return $f(7);
}

echo applyAX(static fn(int $n): string => (string) $n);
