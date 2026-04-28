<?php

declare(strict_types=1);

/**
 * @param non-empty-list<int> $xs
 */
function shift_first(array $xs): int
{
    $first = array_shift($xs);
    return $first;
}
