<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return list<int>
 */
function compact_filter(array $xs): array
{
    return array_values(array_filter($xs, static fn(int $x): bool => $x > 0));
}
