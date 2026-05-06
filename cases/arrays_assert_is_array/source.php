<?php

declare(strict_types=1);

function describe(mixed $x): string
{
    if (is_array($x)) {
        return 'array of size ' . (string) count($x);
    }
    return 'not-array';
}
