<?php

declare(strict_types=1);

/** @param numeric-string $s */
function takes_numeric(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    if (is_numeric($s)) {
        takes_numeric($s);
    }
}
