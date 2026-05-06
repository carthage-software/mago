<?php

declare(strict_types=1);

function callables_takes_one(string $a): string
{
    return $a;
}

callables_takes_one(...['a', 'b', 'c']);
