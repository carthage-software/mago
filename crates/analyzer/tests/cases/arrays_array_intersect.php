<?php

declare(strict_types=1);

/**
 * @param list<int> $a
 * @param list<int> $b
 * @return array<int, int>
 */
function intersect(array $a, array $b): array
{
    return array_intersect($a, $b);
}
