<?php

declare(strict_types=1);

/**
 * @param array<string, ?int> $arr
 */
function get_or_default(array $arr, string $key): ?int
{
    if (array_key_exists($key, $arr)) {
        return $arr[$key];
    }
    return null;
}
