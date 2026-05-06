<?php

declare(strict_types=1);

/**
 * @param list<int> $a
 * @param list<int> $b
 * @return array<int, int>
 */
function diff(array $a, array $b): array
{
    return array_diff($a, $b);
}
