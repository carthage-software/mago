<?php

declare(strict_types=1);

function callables_takes_int_only(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
callables_takes_int_only(fn(): int => 1);
