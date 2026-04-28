<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @param list<int> $ys
 * @return list<int>
 */
function zip_add(array $xs, array $ys): array
{
    return array_map(static fn(int $a, int $b): int => $a + $b, $xs, $ys);
}
