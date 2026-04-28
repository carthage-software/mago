<?php

declare(strict_types=1);

function callables_int_target(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
callables_int_target(fn(): int => 1);
