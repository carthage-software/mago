<?php

declare(strict_types=1);

function callables_with_default(int $a, int $b = 10): int
{
    return $a + $b;
}

$x = callables_with_default(1);
$y = callables_with_default(1, 2);
echo $x + $y;
