<?php

declare(strict_types=1);

/** @param uppercase-string $s */
function takes_uppercase(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    takes_uppercase($s);
}
