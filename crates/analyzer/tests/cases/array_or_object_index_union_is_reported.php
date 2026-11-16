<?php

declare(strict_types=1);

/**
 * @param array<string, int> $values
 * @param array<array-key, mixed>|string $key
 */
function read_with_array_key(array $values, array|string $key): int
{
    return $values[$key]; // @mago-expect analysis:mismatched-array-index
}

/**
 * @param list<string> $values
 */
function read_with_object_key(array $values, int|stdClass $key): string
{
    return $values[$key]; // @mago-expect analysis:mismatched-array-index
}

/**
 * @param array<array-key, mixed>|int|string $key
 *
 * @return array<array-key, int>
 */
function write_with_array_key(array|int|string $key): array
{
    $values = [];
    $values[$key] = 1; // @mago-expect analysis:mismatched-array-index

    return $values;
}

/**
 * @param array<int, string> $values
 */
function read_with_wider_key(array $values, int|string $key): string
{
    return $values[$key] ?? '';
}
