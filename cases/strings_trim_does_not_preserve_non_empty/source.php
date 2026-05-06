<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

/** @param non-empty-string $s */
function probe(string $s): void
{
    takes_non_empty(trim($s));
}
