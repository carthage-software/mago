<?php

declare(strict_types=1);

/** @param interface-string $c */
function takes_interface_string(string $c): void
{
    echo $c;
}

function probe(string $c): void
{
    if (interface_exists($c)) {
        takes_interface_string($c);
    }
}
