<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    $s .= 'x';
    takes_non_empty($s);
}
