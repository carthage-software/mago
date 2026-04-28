<?php

declare(strict_types=1);

function callables_str_only(string $s): int
{
    return strlen($s);
}

$cb = callables_str_only(...);
/** @mago-expect analysis:invalid-argument */
$cb(42);
