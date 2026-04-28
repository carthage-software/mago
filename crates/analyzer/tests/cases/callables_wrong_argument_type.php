<?php

declare(strict_types=1);

function callables_takes_int(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
callables_takes_int('not an int');
