<?php

declare(strict_types=1);

function probe(): void
{
    $a = [1, 2];

    /** @mago-expect analysis:array-to-string-conversion */
    $a .= 'x';
}
