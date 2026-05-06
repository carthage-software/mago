<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param iterable<T> $items
 *
 * @return list<T>
 */
function gen_to_list2(iterable $items): array
{
    $r = [];
    foreach ($items as $i) {
        $r[] = $i;
    }
    return $r;
}

/**
 * @param list<int> $a
 */
function take_list_int(array $a): void
{
    foreach ($a as $n) {
        echo $n;
    }
}

take_list_int(gen_to_list2([1, 2, 3]));
