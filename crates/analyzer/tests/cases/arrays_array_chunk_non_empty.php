<?php

declare(strict_types=1);

/**
 * @param list<int> $ids
 *
 * @return list<non-empty-list<int>>
 */
function batches(array $ids): array
{
    return array_chunk($ids, 100);
}

/**
 * @param array<string, int> $map
 *
 * @return list<non-empty-array<string, int>>
 */
function batches_with_keys(array $map): array
{
    return array_chunk($map, 100, true);
}
