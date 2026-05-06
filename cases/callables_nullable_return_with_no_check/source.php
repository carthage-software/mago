<?php

declare(strict_types=1);

function callables_maybe_int(bool $on): ?int
{
    return $on ? 1 : null;
}

function callables_consume_int(int $n): void
{
    echo $n;
}

callables_consume_int(callables_maybe_int(true));
