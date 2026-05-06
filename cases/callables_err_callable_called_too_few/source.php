<?php

declare(strict_types=1);

/**
 * @param callable(int, int): int $cb
 */
function callables_call_two(callable $cb): int
{
    return $cb(1);
}

callables_call_two(fn(int $a, int $b): int => $a + $b);
