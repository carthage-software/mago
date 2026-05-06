<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function get_or_default(array $arr, string $key): int
{
    if (isset($arr[$key])) {
        return $arr[$key];
    }
    return 0;
}
