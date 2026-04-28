<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function takes_non_empty(string $s): void
{
    echo $s;
}

function check(string $s): void
{
    if ($s !== '') {
        takes_non_empty($s);
    }
}

function check_flipped(string $s): void
{
    if ('' !== $s) {
        takes_non_empty($s);
    }
}
