<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return non-empty-list<int>
 */
function append_one(array $xs): array
{
    $xs[] = 1;
    return $xs;
}
