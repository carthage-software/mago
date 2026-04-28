<?php

declare(strict_types=1);

/**
 * @param array<string, int> $map
 */
function flow_negated_array_key_exists(array $map, string $key): int
{
    if (!array_key_exists($key, $map)) {
        return -1;
    }

    return $map[$key];
}
