<?php

declare(strict_types=1);

/**
 * @param list<list<int>> $matrix
 */
function deep_count(array $matrix): int
{
    return count($matrix, COUNT_RECURSIVE);
}
