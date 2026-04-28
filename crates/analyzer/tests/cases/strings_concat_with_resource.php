<?php

declare(strict_types=1);

function probe(): string
{
    $r = fopen('/dev/null', 'r');

    /** @mago-expect analysis:implicit-resource-to-string-cast */
    return 'foo' . $r;
}
