<?php

declare(strict_types=1);

/**
 * @param callable(int): string $cb
 */
function callables_invoke_callable(callable $cb): string
{
    return $cb(42);
}

callables_invoke_callable(fn(int $n): string => (string) $n);
