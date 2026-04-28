<?php

declare(strict_types=1);

/**
 * @param non-empty-list<int> $keys
 * @param non-empty-list<string> $values
 * @return non-empty-array<int, string>
 */
function combine_strict(array $keys, array $values): array
{
    return array_combine($keys, $values);
}
