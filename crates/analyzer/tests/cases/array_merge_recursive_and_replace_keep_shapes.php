<?php

declare(strict_types=1);

/**
 * @param array{name: string} $a
 * @param array{age: int} $b
 *
 * @return array{name: string, age: int}
 */
function merge_recursive_disjoint(array $a, array $b): array
{
    return array_merge_recursive($a, $b);
}

/**
 * @param array{name: string, age: string} $a
 * @param array{age: int} $b
 *
 * @return array{name: string, age: int}
 */
function replace_shapes(array $a, array $b): array
{
    return array_replace($a, $b);
}

/**
 * @param array<string, int> $x
 * @param array<string, int> $y
 *
 * @return array<string, int>
 */
function merge_recursive_shared(array $x, array $y): array
{
    // @mago-expect analysis:invalid-return-statement
    return array_merge_recursive($x, $y);
}

/**
 * @return array{a: non-empty-list<int>, b: string}
 */
function merge_recursive_gathers(): array
{
    return array_merge_recursive(['a' => 1, 'b' => 's'], ['a' => 2]);
}

/**
 * @param array<string, list<int>> $x
 * @param array<string, list<int>> $y
 *
 * @return array<string, list<int>>
 */
function merge_recursive_lists(array $x, array $y): array
{
    return array_merge_recursive($x, $y);
}
