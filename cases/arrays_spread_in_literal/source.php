<?php

declare(strict_types=1);

/**
 * @param list<int> $a
 * @param list<int> $b
 * @return list<int>
 */
function combine(array $a, array $b): array
{
    return [...$a, ...$b];
}
