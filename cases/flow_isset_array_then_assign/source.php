<?php

declare(strict_types=1);

/**
 * @param array<string, int> $a
 */
function flow_isset_array_then_assign(array $a, string $key): int
{
    if (!isset($a[$key])) {
        $a[$key] = 0;
    }

    return $a[$key] + 1;
}
