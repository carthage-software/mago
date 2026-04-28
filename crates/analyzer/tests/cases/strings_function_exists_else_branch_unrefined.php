<?php

declare(strict_types=1);

/** @param callable-string $c */
function takes_cs(string $c): void
{
    $c();
}

function probe(string $c): void
{
    if (!function_exists($c)) {
        /** @mago-expect analysis:possibly-invalid-argument */
        takes_cs($c);
    }
}
