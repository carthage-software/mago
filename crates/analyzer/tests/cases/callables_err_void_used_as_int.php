<?php

declare(strict_types=1);

function callables_returns_void_two(): void
{
}

function callables_takes_int_again(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
callables_takes_int_again(callables_returns_void_two());
