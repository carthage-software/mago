<?php

declare(strict_types=1);

/**
 * @param callable(int): int $cb
 */
function callables_int_int_only(callable $cb): int
{
    return $cb(1);
}

/** @mago-expect analysis:invalid-argument */
callables_int_int_only(fn(int $n, int $m): int => $n + $m);
