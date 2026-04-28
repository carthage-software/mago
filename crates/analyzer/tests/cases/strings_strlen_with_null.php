<?php

declare(strict_types=1);

function probe(): int
{
    /** @mago-expect analysis:null-argument */
    return strlen(null);
}
