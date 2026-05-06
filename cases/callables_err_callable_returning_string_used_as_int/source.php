<?php

declare(strict_types=1);

function callables_takes_int_only(int $n): int
{
    return $n;
}

/** @param callable(): string $cb */
function callables_run_str_returning_cb(callable $cb): int
{
    return callables_takes_int_only($cb());
}

callables_run_str_returning_cb(fn(): string => 'hi');
