<?php

declare(strict_types=1);

function probe(): string
{
    $r = fopen('/dev/null', 'r');

    return 'foo' . $r;
}
