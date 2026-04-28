<?php

declare(strict_types=1);

/** @param numeric-string $s */
function takes_numeric(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_numeric($s);
}
