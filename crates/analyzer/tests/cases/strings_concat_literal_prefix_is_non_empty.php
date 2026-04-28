<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(string $tail): void
{
    takes_non_empty('prefix_' . $tail);
}

function probe_suffix(string $head): void
{
    takes_non_empty($head . '_suffix');
}
