<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return list<int>
 */
function take_first_three(array $xs): array
{
    return array_slice($xs, 0, 3);
}
