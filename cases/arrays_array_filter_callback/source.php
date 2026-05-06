<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return array<int, int>
 */
function only_positive(array $xs): array
{
    return array_filter($xs, static fn(int $x): bool => $x > 0);
}
