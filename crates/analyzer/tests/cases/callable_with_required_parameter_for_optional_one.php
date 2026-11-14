<?php

declare(strict_types=1);

/**
 * @param callable(string=): string $f
 */
function map(callable $f): string
{
    return $f();
}

function test(): void
{
    echo map(fn(string $s = 'x'): string => $s);
    echo map(fn(string ...$s): string => implode('', $s));
    // @mago-expect analysis:invalid-argument
    echo map(fn(string $s): string => $s);
}
