<?php

declare(strict_types=1);

function callables_run_nullable_cb(?callable $cb): mixed
{
    return $cb(1);
}

callables_run_nullable_cb(fn(int $n): int => $n);
