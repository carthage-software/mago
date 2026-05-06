<?php

declare(strict_types=1);

function callables_join(string $glue, string $a, string $b = '', string $c = ''): string
{
    return $a . $glue . $b . $glue . $c;
}

echo callables_join(',', 'one');
echo callables_join(',', 'one', 'two');
echo callables_join(',', 'one', 'two', 'three');
