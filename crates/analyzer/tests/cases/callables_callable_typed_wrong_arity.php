<?php

declare(strict_types=1);

/**
 * @param callable(int): string $cb
 */
function callables_run_cb(callable $cb): string
{
    /** @mago-expect analysis:too-many-arguments */
    return $cb(1, 2);
}

callables_run_cb(fn(int $n): string => (string) $n);
