<?php

declare(strict_types=1);

/**
 * @param list<string> $parts
 */
function probe(array $parts): string
{
    return implode(',', $parts);
}
