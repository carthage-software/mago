<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T|list<T> $value
 *
 * @return list<T>
 */
function wrap(mixed $value): array
{
    exit(1);
}

/**
 * @template T
 *
 * @param T|array<array-key, mixed> $value
 *
 * @return T
 */
function not_array(mixed $value): mixed
{
    return is_array($value) ? exit(1) : $value;
}

/**
 * @param int|list<int> $ids
 *
 * @return list<int>
 */
function ids(int|array $ids): array
{
    return wrap($ids);
}

/**
 * @param array<string>|int $in
 */
function number(array|int $in): int
{
    return not_array($in);
}

/**
 * @param list<int> $ids
 *
 * @return list<int>
 */
function only_lists(array $ids): array
{
    return wrap($ids);
}
