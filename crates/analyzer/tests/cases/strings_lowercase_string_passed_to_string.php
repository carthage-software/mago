<?php

declare(strict_types=1);

function takes_string(string $s): void
{
    echo $s;
}

/** @param lowercase-string $s */
function probe(string $s): void
{
    takes_string($s);
}
