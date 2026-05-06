<?php

declare(strict_types=1);

/** @param trait-string $c */
function takes_trait_string(string $c): void
{
    echo $c;
}

function probe(string $c): void
{
    if (trait_exists($c)) {
        takes_trait_string($c);
    }
}
