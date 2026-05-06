<?php

declare(strict_types=1);

function callables_takes_strings(string $a, string $b): string
{
    return $a . $b;
}

callables_takes_strings(...['a', 1]);
