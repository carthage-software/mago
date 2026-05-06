<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function reverse_sort(array $xs): void
{
    usort($xs, static fn(int $a, int $b): int => $b <=> $a);
}
