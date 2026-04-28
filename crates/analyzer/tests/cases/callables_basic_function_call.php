<?php

declare(strict_types=1);

function callables_add(int $a, int $b): int
{
    return $a + $b;
}

$result = callables_add(1, 2);
echo $result;
