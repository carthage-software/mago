<?php

declare(strict_types=1);

/**
 * @param callable(int): int $cb
 */
function callables_call_one_int(callable $cb): int
{
    /** @mago-expect analysis:too-many-arguments */
    return $cb(1, 2);
}

callables_call_one_int(fn(int $n): int => $n);
