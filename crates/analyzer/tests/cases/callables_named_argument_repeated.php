<?php

declare(strict_types=1);

function callables_simple_a_b(int $a, int $b): int
{
    return $a + $b;
}

/** @mago-expect analysis:duplicate-named-argument */
callables_simple_a_b(a: 1, a: 2);
