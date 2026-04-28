<?php

declare(strict_types=1);

/**
 * @param list<string> $keys
 * @return array<string, int>
 */
function fill_keys(array $keys): array
{
    return array_fill_keys($keys, 0);
}
