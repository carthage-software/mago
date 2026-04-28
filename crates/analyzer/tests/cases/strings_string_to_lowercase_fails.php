<?php

declare(strict_types=1);

/** @param lowercase-string $s */
function takes_lowercase(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_lowercase($s);
}
