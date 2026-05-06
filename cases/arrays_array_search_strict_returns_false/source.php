<?php

declare(strict_types=1);

/**
 * @param list<string> $xs
 * @return int|false
 */
function find(array $xs, string $needle): int|false
{
    return array_search($needle, $xs, true);
}
