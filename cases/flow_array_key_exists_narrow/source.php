<?php

declare(strict_types=1);

/**
 * @param array<string, int> $map
 */
function flow_array_key_exists_narrow(array $map, string $key): int
{
    if (array_key_exists($key, $map)) {
        return $map[$key];
    }

    return 0;
}
