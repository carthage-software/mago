<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return list<int>
 */
function cut_first_two(array $xs): array
{
    return array_splice($xs, 0, 2);
}

/**
 * @param array<string, int> $map
 * @return int
 */
function cut_and_sum(array $map): int
{
    $removed = array_splice($map, 0, 2);

    $total = 0;
    foreach ($removed as $value) {
        $total += $value;
    }

    return $total;
}
