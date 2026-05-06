<?php

declare(strict_types=1);

/**
 * @param list<int> $a
 * @param list<int> $b
 * @return list<int>
 */
function merge_lists(array $a, array $b): array
{
    return array_merge($a, $b);
}
