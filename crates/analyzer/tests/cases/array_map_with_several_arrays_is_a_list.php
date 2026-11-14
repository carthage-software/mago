<?php

declare(strict_types=1);

/**
 * @param array<string, int> $x
 * @param array<string, int> $y
 * @return list<int>
 */
function sum(array $x, array $y): array
{
    $r = array_map(static fn(int $a, int $b): int => $a + $b, $x, $y);

    return $r;
}
