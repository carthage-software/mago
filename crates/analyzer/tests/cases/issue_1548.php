<?php

declare(strict_types=1);

/**
 * @param list<list<int>> $a
 * @return list<int>
 */
function spread_lists(array $a): array
{
    return array_merge(...$a);
}

/**
 * @param list<non-empty-list<int>> $a
 * @return non-empty-list<int>
 */
function spread_non_empty_inner(array $a): array
{
    return array_merge(...$a);
}

/**
 * @param list<array<string, int>> $a
 * @return array<string, int>
 */
function spread_keyed(array $a): array
{
    return array_merge(...$a);
}
