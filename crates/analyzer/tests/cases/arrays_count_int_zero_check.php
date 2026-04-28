<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function maybe_first(array $xs): ?int
{
    if (count($xs) > 0) {
        return $xs[0];
    }
    return null;
}
