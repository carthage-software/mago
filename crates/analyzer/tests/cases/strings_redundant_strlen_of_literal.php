<?php

declare(strict_types=1);

function probe(): bool
{
    /** @mago-expect analysis:redundant-comparison */
    return strlen('hello') === 99;
}
