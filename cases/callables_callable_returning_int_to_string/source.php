<?php

declare(strict_types=1);

function callables_needs_string_input(string $s): void
{
    echo $s;
}

/**
 * @param callable(int): int $cb
 */
function callables_run_cb_int(callable $cb): void
{
    callables_needs_string_input($cb(1));
}

callables_run_cb_int(fn(int $n): int => $n + 1);
