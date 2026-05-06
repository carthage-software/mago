<?php

declare(strict_types=1);

function callables_named_spread(string $first, int $second): string
{
    return $first . $second;
}

$args = ['first' => 'hi', 'second' => 7];
echo callables_named_spread(...$args);
