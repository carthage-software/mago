<?php

declare(strict_types=1);

/** @param '' $s */
function takes_empty(string $s): void
{
    echo $s;
}

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    if ($s === '') {
        takes_empty($s);
    } else {
        takes_non_empty($s);
    }
}
