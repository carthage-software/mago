<?php

declare(strict_types=1);

function callables_two_a_b(string $a, string $b): string
{
    return $a . $b;
}

callables_two_a_b(a: 'x', b: 'y', a: 'z'); // @mago-expect analysis:duplicate-named-argument,too-many-arguments
