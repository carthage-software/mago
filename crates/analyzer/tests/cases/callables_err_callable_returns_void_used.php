<?php

declare(strict_types=1);

function callables_void_returner(): void
{
    echo 'done';
}

function callables_takes_int_arg(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
callables_takes_int_arg(callables_void_returner());
