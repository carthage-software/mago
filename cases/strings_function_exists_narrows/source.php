<?php

declare(strict_types=1);

/** @param callable-string $c */
function takes_callable_string(string $c): void
{
    $c();
}

function probe(string $c): void
{
    if (function_exists($c)) {
        takes_callable_string($c);
    }
}
