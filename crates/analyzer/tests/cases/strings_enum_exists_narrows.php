<?php

declare(strict_types=1);

/** @param enum-string $c */
function takes_enum_string(string $c): void
{
    echo $c;
}

function probe(string $c): void
{
    if (enum_exists($c)) {
        takes_enum_string($c);
    }
}
