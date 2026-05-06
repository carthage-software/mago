<?php

declare(strict_types=1);

function callables_two_required_three(string $a, int $b): string
{
    return $a . $b;
}

callables_two_required_three(a: 'x');
