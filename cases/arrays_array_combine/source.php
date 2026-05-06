<?php

declare(strict_types=1);

/**
 * @param list<string> $keys
 * @param list<int> $values
 * @return array<string, int>
 */
function combine_arrays(array $keys, array $values): array
{
    return array_combine($keys, $values);
}
