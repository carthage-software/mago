<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(string $a, string $b): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_non_empty($a . $b);
}
