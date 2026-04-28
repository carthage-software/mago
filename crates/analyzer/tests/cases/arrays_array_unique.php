<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return array<int, int>
 */
function dedup(array $xs): array
{
    return array_unique($xs);
}
