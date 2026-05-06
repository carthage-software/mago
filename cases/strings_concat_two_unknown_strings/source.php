<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(string $a, string $b): void
{
    takes_non_empty($a . $b);
}
