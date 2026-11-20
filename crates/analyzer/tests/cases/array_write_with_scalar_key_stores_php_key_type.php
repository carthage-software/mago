<?php

declare(strict_types=1);

/**
 * @param scalar $key
 */
function count_after_scalar_key_write($key): int
{
    $values = [];
    $values[$key] = 4;

    return count($values);
}

/**
 * @param scalar $key
 *
 * @return array<array-key, int>
 */
function return_after_scalar_key_write($key): array
{
    $values = [];
    $values[$key] = 4;

    return $values;
}
