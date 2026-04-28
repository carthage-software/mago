<?php

declare(strict_types=1);

/**
 * @template V
 *
 * @param iterable<V> $items
 *
 * @return list<V>
 */
function gen_collect(iterable $items): array
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
function takes_int_list(array $a): void
{
}

/** @mago-expect analysis:invalid-argument */
takes_int_list(gen_collect(['a', 'b']));
