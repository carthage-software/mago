<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function probe(string $s): void
{
    if ($s) {
        takes_non_empty($s);
    }
}

function probe_else(string $s): void
{
    if (!$s) {
        return;
    }

    takes_non_empty($s);
}
