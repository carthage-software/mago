<?php

declare(strict_types=1);

function probe(): string
{
    $a = [1, 2, 3];

    /** @mago-expect analysis:array-to-string-conversion */
    return "values: $a";
}
