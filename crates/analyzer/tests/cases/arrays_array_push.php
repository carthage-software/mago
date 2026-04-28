<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return non-empty-list<int>
 */
function push_one(array $xs): array
{
    array_push($xs, 1);
    return $xs;
}
