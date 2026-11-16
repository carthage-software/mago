<?php

declare(strict_types=1);

const COLUMNS = ['a' => 1, 'b' => 2];

/**
 * @param array<string, string|null> $data
 *
 * @return array{'a'?: string, 'b'?: string}
 */
function provided_columns(array $data): array
{
    $provided = [];
    foreach (COLUMNS as $column => $_) {
        if (null === ($data[$column] ?? null)) {
            continue;
        }

        $provided[$column] = $data[$column];
    }

    return $provided;
}

/**
 * @param array<string, string|null> $data
 */
function first_column(array $data): string
{
    $provided = provided_columns($data);
    $first = array_key_first($provided);
    if (null === $first) {
        return '';
    }

    return $provided[$first];
}

/**
 * @param array<string, string|null> $data
 */
function last_column(array $data): string
{
    $provided = provided_columns($data);
    $last = array_key_last($provided);
    if ($last !== null) {
        return $provided[$last];
    }

    return '';
}
