<?php

declare(strict_types=1);

function callables_int_only(int $n): int
{
    return $n;
}

$cb = callables_int_only(...);
/** @mago-expect analysis:invalid-argument */
$cb('bad');
