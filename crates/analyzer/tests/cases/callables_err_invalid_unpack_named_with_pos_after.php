<?php

declare(strict_types=1);

function callables_target_str_int(string $a, int $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:invalid-argument */
callables_target_str_int(...['a' => 'x', 'b' => 'wrong']);
