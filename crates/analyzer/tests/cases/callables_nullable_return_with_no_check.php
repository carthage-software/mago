<?php

declare(strict_types=1);

function callables_maybe_int(bool $on): null|int
{
    return $on ? 1 : null;
}

function callables_consume_int(int $n): void
{
    echo $n;
}

/** @mago-expect analysis:possibly-null-argument */
callables_consume_int(callables_maybe_int(true));
