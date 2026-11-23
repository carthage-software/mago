<?php

declare(strict_types=1);

/**
 * @param array<string, mixed> $query
 */
function assigned_array_key_replaces_condition_narrowing(array $query): int
{
    $query['id'] = is_scalar($query['id'] ?? null) ? 1 : 0;

    if (!empty($query['id'])) {
        return 1;
    }

    return $query['id'];
}
