<?php

declare(strict_types=1);

/** @param non-empty-lowercase-string $s */
function takes_ne_lc(string $s): void
{
    echo $s;
}

/** @param non-empty-string $s */
function probe(string $s): void
{
    takes_ne_lc(strtolower($s));
}
