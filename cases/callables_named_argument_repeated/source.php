<?php

declare(strict_types=1);

function callables_simple_a_b(int $a, int $b): int
{
    return $a + $b;
}

callables_simple_a_b(a: 1, a: 2);
