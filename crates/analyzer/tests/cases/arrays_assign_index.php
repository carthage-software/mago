<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return array<string, int>
 */
function set_key(array $arr): array
{
    $arr['x'] = 42;
    return $arr;
}
