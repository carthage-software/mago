<?php

declare(strict_types=1);

function takes_string(string $s): void
{
    echo $s;
}

function probe(int|string $val): void
{
    if (is_string($val)) {
        takes_string($val);
    }
}
