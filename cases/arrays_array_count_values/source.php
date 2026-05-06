<?php

declare(strict_types=1);

/**
 * @param list<string> $xs
 * @return array<string, int>
 */
function frequencies(array $xs): array
{
    return array_count_values($xs);
}
