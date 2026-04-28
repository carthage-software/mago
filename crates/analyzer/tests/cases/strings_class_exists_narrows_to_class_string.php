<?php

declare(strict_types=1);

/** @param class-string $c */
function takes_class_string(string $c): void
{
    echo $c;
}

function probe(string $c): void
{
    if (class_exists($c)) {
        takes_class_string($c);
    }
}
