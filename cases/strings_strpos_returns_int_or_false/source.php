<?php

declare(strict_types=1);

function probe(string $h): bool
{
    return strpos($h, 'x') !== false;
}
