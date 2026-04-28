<?php

declare(strict_types=1);

/**
 * @param non-empty-list<int> $xs
 */
function pop_last(array $xs): int
{
    $last = array_pop($xs);
    return $last;
}
