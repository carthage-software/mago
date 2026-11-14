<?php

declare(strict_types=1);

/**
 * @param list<array{0: string, 1: int}> $rows
 * @return array<string, int>
 */
function indexed(array $rows): array
{
    return array_column($rows, 1, 0);
}

/**
 * @param list<array{0: string, 1: int}> $rows
 * @return list<int>
 */
function column(array $rows): array
{
    return array_column($rows, 1);
}

/**
 * @param list<array{0: string, 1: int}> $rows
 * @return array<string, array{0: string, 1: int}>
 */
function rows_by_key(array $rows): array
{
    return array_column($rows, null, 0);
}
