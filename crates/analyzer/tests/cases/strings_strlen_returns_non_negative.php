<?php

declare(strict_types=1);

function probe(string $s): bool
{
    /** @mago-expect analysis:redundant-comparison */
    return strlen($s) < 0;
}
