<?php

declare(strict_types=1);

function callables_uses_func_get_args(int $a, int $b, int $c): int
{
    $args = func_get_args();
    return count($args);
}

echo callables_uses_func_get_args(1, 2, 3);
