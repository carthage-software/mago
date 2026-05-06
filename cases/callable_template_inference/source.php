<?php

/**
 * @template T
 * @param iterable<T> $items
 * @param callable(T): bool $predicate
 * @return list<T>
 */
function list_filter(iterable $items, callable $predicate): array
{
    $result = [];
    foreach ($items as $item) {
        if ($predicate($item)) {
            $result[] = $item;
        }
    }

    return $result;
}

$numbers = [1, 2, 3, 4, 5];
$evens = list_filter($numbers, fn($n) => ($n % 2) === 0);
