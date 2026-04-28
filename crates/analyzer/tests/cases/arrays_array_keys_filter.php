<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return list<string>
 */
function keys_where_42(array $arr): array
{
    return array_keys($arr, 42, true);
}
