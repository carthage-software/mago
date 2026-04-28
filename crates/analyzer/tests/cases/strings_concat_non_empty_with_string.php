<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

/** @param non-empty-string $head */
function probe(string $head, string $tail): void
{
    takes_non_empty($head . $tail);
}

/** @param non-empty-string $tail */
function probe_tail(string $head, string $tail): void
{
    takes_non_empty($head . $tail);
}
