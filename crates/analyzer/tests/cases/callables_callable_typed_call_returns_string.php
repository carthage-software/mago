<?php

declare(strict_types=1);

function callables_takes_string(string $s): void
{
    echo $s;
}

/**
 * @param callable(int): string $cb
 */
function callables_use_cb(callable $cb): void
{
    callables_takes_string($cb(42));
}

callables_use_cb(fn(int $n): string => (string) $n);
