<?php

declare(strict_types=1);

/**
 * @param callable(string): int $cb
 */
function callables_run_str_cb(callable $cb): int
{
    return $cb('hi');
}

callables_run_str_cb(fn(string $s): int => strlen($s));
