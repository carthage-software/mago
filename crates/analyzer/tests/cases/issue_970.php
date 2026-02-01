<?php

declare(strict_types=1);

/**
 * @param list<int> $list
 *
 * @return non-empty-list<int>
 */
function test_array_unshift_preserves_list(array $list): array
{
    array_unshift($list, 17);

    // $list should be non-empty-list<int>, not array<non-negative-int, int>
    return $list;
}

/**
 * @param list<int> $list
 *
 * @return non-empty-list<int>
 */
function test_array_push_preserves_list(array $list): array
{
    array_push($list, 17);

    // $list should be non-empty-list<int>
    return $list;
}
