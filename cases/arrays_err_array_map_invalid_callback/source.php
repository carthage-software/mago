<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad(array $xs): array
{
    return array_map(42, $xs);
}
