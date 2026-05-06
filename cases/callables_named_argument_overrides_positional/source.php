<?php

declare(strict_types=1);

function callables_collide(string $a, int $b): string
{
    return $a . $b;
}

callables_collide('x', 1, a: 'y');
