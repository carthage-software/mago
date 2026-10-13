<?php

declare(strict_types=1);

namespace Issue2485;

/** @param array<array-key, int|string> $rule */
function match_or_none(array $rule): string
{
    ['match' => $match] = $rule + ['match' => null];

    return is_string($match) ? $match : 'none';
}

/**
 * @param array{name?: int} $left
 * @param array<array-key, string> $right
 */
function optional_string_key(array $left, array $right): bool
{
    $merged = $left + $right;

    if (isset($merged['name']) && is_string($merged['name'])) {
        return true;
    }

    return false;
}

/**
 * @param array{0?: int} $left
 * @param array<int, string> $right
 */
function optional_int_key(array $left, array $right): bool
{
    $merged = $left + $right;

    if (isset($merged[0])) {
        return true;
    }

    return false;
}

/** @param array<int, int> $ints */
function string_key_on_int_keys(array $ints): ?string
{
    ['name' => $name] = $ints + ['name' => null];

    return $name;
}
