<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 *
 * @return list<int>
 */
function pad_to_five(array $xs): array
{
    return array_pad($xs, 5, 0);
}

/**
 * @param list<int> $xs
 *
 * @return list<int>
 */
function pad_left(array $xs): array
{
    return array_pad($xs, -5, 0);
}

/**
 * @param array<string, int> $assoc
 *
 * @return array<array-key, int>
 */
function pad_assoc(array $assoc): array
{
    return array_pad($assoc, 5, 0);
}
