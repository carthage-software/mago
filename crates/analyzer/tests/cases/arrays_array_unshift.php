<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return non-empty-list<int>
 */
function prepend_one(array $xs): array
{
    array_unshift($xs, 1);
    return $xs;
}
