<?php

declare(strict_types=1);

function callables_one_arg_only(string $s): int
{
    return strlen($s);
}

$cb = callables_one_arg_only(...);
/** @mago-expect analysis:too-many-arguments */
$cb('a', 'b');
