<?php

declare(strict_types=1);

function callables_int_only(int $n): int
{
    return $n;
}

$cb = callables_int_only(...);
$cb('bad');
