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
function gen_list_map_2(array $list, callable $fn): array
{
    $out = [];
    foreach ($list as $item) {
        $out[] = $fn($item);
    }
    return $out;
}

/** @mago-expect analysis:invalid-argument */
gen_list_map_2([1, 2, 3], static fn(string $s): string => $s);
