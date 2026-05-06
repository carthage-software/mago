<?php

declare(strict_types=1);

function callables_target_str_int(string $a, int $b): string
{
    return $a . $b;
}

callables_target_str_int(...['a' => 'x', 'b' => 'wrong']);
