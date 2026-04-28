<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return list<int>
 */
function double_all(array $xs): array
{
    return array_map(static fn(int $x): int => $x * 2, $xs);
}
