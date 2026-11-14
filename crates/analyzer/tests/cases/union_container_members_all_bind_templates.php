<?php

declare(strict_types=1);

/**
 * @template K of array-key
 * @template V
 *
 * @param Traversable<K, V>|array<K, V> $items
 *
 * @return list<V>
 */
function values(Traversable|array $items): array
{
    $values = [];
    foreach ($items as $item) {
        $values[] = $item;
    }

    return $values;
}

/**
 * @param Traversable<int, string>|array<string, bool> $items
 *
 * @return list<bool|string>
 */
function through_union(Traversable|array $items): array
{
    return values($items);
}

/**
 * @param Traversable<int, string>|array<string, bool> $items
 *
 * @return array<int|string, bool|string>
 */
function through_native(Traversable|array $items): array
{
    return iterator_to_array($items);
}

/**
 * @param Traversable<int, string>|array<string, bool> $items
 *
 * @return list<string>
 */
function wrong(Traversable|array $items): array
{
    // @mago-expect analysis:invalid-return-statement
    return values($items);
}

/**
 * @template K of array-key
 * @template V
 *
 * @param iterable<K, V> $items
 *
 * @return array<K, V>
 */
function collect(iterable $items): array
{
    $all = [];
    foreach ($items as $key => $item) {
        $all[$key] = $item;
    }

    return $all;
}

/**
 * @param Traversable<int, string>|array<string, bool> $items
 *
 * @return array<int|string, bool|string>
 */
function through_iterable(Traversable|array $items): array
{
    return collect($items);
}
