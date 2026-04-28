<?php

declare(strict_types=1);

function probe(): string
{
    $arr = [1, 2, 3];

    /** @mago-expect analysis:array-to-string-conversion */
    return 'foo' . $arr;
}
