<?php

declare(strict_types=1);

/**
 * @param list<int> $x
 * @param list<int> $y
 * @return list<array<array-key, int>>
 */
function zip(array $x, array $y): array
{
    return array_map(static fn(int ...$values): array => $values, $x, $y);
}

/**
 * @param list<int> $x
 * @param list<int> $y
 * @return list<list{int, int}>
 */
function pair(array $x, array $y): array
{
    return array_map(static fn(int $a, int $b): array => [$a, $b], $x, $y);
}
