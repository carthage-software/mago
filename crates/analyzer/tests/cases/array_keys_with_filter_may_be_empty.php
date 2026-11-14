<?php

declare(strict_types=1);

/**
 * @param non-empty-array<string, int> $map
 */
function array_keys_filter_has_value(array $map, int $value): bool
{
    $keys = array_keys($map, $value, true);

    if (count($keys) === 0) {
        return false;
    }

    return true;
}

