<?php

declare(strict_types=1);

/**
 * @template T of non-empty-array<string, int>
 *
 * @param T $values
 */
function first_of_bound(array $values): int
{
    return array_first($values);
}

/**
 * @template T of list<int>
 *
 * @param T $values
 *
 * @return list<int>
 */
function values_of_bound(array $values): array
{
    return array_values($values);
}

/**
 * @template T of non-empty-list<int>
 *
 * @param T $values
 */
function pop_of_bound(array $values): int
{
    return array_pop($values);
}

/**
 * @template V
 *
 * @param array<array-key, V> $values
 *
 * @return V|null
 */
function my_first(array $values): mixed
{
    foreach ($values as $value) {
        return $value;
    }

    return null;
}

/**
 * @template T of array<string, int>
 *
 * @param T $values
 */
function user_generic(array $values): ?int
{
    return my_first($values);
}

/**
 * @template T of list<int>
 *
 * @param T $values
 *
 * @return list<string>
 */
function map_of_bound(array $values): array
{
    return array_map(static fn(int $value): string => (string) $value, $values);
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return T
 */
function identity(mixed $value): mixed
{
    return $value;
}

/**
 * @template T of list<int>
 *
 * @param T $values
 *
 * @return T
 */
function keeps_bare_template(array $values): array
{
    return identity($values);
}
