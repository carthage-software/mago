<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return array<int, int>
 */
function only_truthy(array $xs): array
{
    return array_filter($xs, 'boolval');
}
