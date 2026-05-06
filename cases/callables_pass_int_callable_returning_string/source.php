<?php

declare(strict_types=1);

function callables_takes_string_only_input(string $s): void
{
    echo $s;
}

/** @param callable(int): string $cb */
function callables_run_typed(callable $cb, int $n): void
{
    callables_takes_string_only_input($cb($n));
}

callables_run_typed(fn(int $n): string => (string) $n, 5);
