<?php

declare(strict_types=1);

/**
 * @param list<string> $xs
 */
function contains(array $xs, string $needle): bool
{
    return in_array($needle, $xs, true);
}
