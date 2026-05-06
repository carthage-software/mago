<?php

declare(strict_types=1);

function probe(): string
{
    $arr = [1, 2, 3];

    return 'foo' . $arr;
}
