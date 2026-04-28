<?php

declare(strict_types=1);

function callables_three(int $a, int $b, int $c): int
{
    return $a + $b + $c;
}

$args = [1, 2, 3];
echo callables_three(...$args);
