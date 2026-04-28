<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(): void
{
    $buf = '';
    $buf .= 'a';
    $buf .= 'b';
    takes_non_empty($buf);
}
