<?php

declare(strict_types=1);

/**
 * @param array<string, int> $map
 */
function flow_isset_array_then_use(array $map, string $key): int
{
    if (isset($map[$key])) {
        return $map[$key] + 1;
    }

    return 0;
}
