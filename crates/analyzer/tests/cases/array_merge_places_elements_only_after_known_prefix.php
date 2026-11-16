<?php

declare(strict_types=1);

/**
 * @param list<string> $values
 */
function take_list(array $values): void
{
}

function built_in_loop(): int
{
    $values = [];
    for ($i = 0; $i <= 5; ++$i) {
        $values = array_merge($values, [$i]);
    }

    return $values[5] ?? -1;
}

/**
 * @param list<string> $list
 */
function after_unknown_length(array $list): bool
{
    $merged = array_merge($list, ['x']);

    return $merged[0] === 'y';
}

/**
 * @param array<int, string> $values
 */
function integer_keys(array $values): void
{
    take_list(array_merge($values, ['x']));
    take_list(array_merge($values, $values));
}

/**
 * @return list{string, string}
 */
function known_lists(): array
{
    return array_merge(['a'], ['b']);
}
