<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 * @return list<int>
 */
function keys_eq_zero(array $xs): array
{
    return array_keys($xs, 0, true);
}
