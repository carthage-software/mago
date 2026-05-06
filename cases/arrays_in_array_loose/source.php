<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function any_loose(array $xs, int $needle): bool
{
    return in_array($needle, $xs);
}
