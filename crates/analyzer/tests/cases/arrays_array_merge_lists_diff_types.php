<?php

declare(strict_types=1);

/**
 * @param list<int> $a
 * @param list<string> $b
 * @return list<int|string>
 */
function merge_diff(array $a, array $b): array
{
    return array_merge($a, $b);
}
