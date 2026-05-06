<?php

declare(strict_types=1);

function callables_takes_two(string $a, int $b): string
{
    return $a . $b;
}

callables_takes_two('hello', 1, 2);
