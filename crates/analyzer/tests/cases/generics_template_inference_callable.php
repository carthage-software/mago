<?php

declare(strict_types=1);

/**
 * @template T
 * @template U
 *
 * @param list<T> $list
 * @param callable(T): U $fn
 *
 * @return list<U>
 */
function gen_list_map(array $list, callable $fn): array
{
    $out = [];
    foreach ($list as $item) {
        $out[] = $fn($item);
    }
    return $out;
}

function takes_list_string(): void
{
    $r = gen_list_map([1, 2, 3], static fn(int $n): string => (string) $n);
    foreach ($r as $s) {
        echo $s;
    }
}
