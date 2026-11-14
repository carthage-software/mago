<?php

declare(strict_types=1);

/**
 * @param array<string, int> $values
 */
function array_merge_unsealed_override(array $values): bool
{
    $merged = array_merge(['foo' => 'foo'], $values);

    return $merged['foo'] === 'foo';
}

/**
 * @param array{foo?: int} $opt
 */
function array_merge_optional_override(array $opt): int|string
{
    $merged = array_merge(['foo' => 'foo'], $opt);

    return $merged['foo'];
}

/**
 * @return array{0: string, 1: string}
 */
function array_merge_renumbers_int_keys(): array
{
    $merged = array_merge([5 => 'a'], [5 => 'b']);

    return [$merged[0], $merged[1]];
}
