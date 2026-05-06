<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return list<list<int>>
 */
function chunk_by_two(array $xs): array
{
    return array_chunk($xs, 2);
}
