<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return non-empty-list<int>
 */
function with_zero(array $xs): array
{
    return [0, ...$xs];
}
