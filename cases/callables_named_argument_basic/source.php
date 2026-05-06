<?php

declare(strict_types=1);

function callables_named(string $first, int $second, bool $third): string
{
    return $first . $second . ($third ? 'y' : 'n');
}

echo callables_named(first: 'a', second: 1, third: true);
echo callables_named(third: false, first: 'b', second: 2);
