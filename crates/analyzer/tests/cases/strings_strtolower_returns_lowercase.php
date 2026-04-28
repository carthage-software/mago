<?php

declare(strict_types=1);

/** @param lowercase-string $s */
function takes_lowercase(string $s): void
{
    echo $s;
}

function probe(string $input): void
{
    takes_lowercase(strtolower($input));
}

function probe_literal(): void
{
    takes_lowercase(strtolower('FOO BAR'));
}
