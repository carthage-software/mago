<?php

declare(strict_types=1);

function call_countable(callable $c): int
{
    if (!is_countable($c)) {
        return 0;
    }

    $c();

    return count($c);
}

/**
 * @template TKey of array-key
 * @template TValue
 *
 * @param iterable<TKey, TValue> $i
 *
 * @return list<TValue>
 */
function values_of_countable_array(iterable $i): array
{
    if (is_countable($i) && is_array($i)) {
        return array_values($i);
    }

    return [];
}

/**
 * @param iterable<mixed, int> $i
 *
 * @return list<int>
 */
function values_of_mixed_key_iterable(iterable $i): array
{
    if (is_countable($i) && is_array($i)) {
        return array_values($i);
    }

    return [];
}
