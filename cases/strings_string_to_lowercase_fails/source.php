<?php

declare(strict_types=1);

/** @param lowercase-string $s */
function takes_lowercase(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    takes_lowercase($s);
}
