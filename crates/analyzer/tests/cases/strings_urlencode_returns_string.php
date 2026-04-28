<?php

declare(strict_types=1);

function probe(string $s): string
{
    return urlencode($s);
}
