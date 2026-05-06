<?php

declare(strict_types=1);

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $arr
 *
 * @return list<V>
 */
function gen_only_values(array $arr): array
{
    return array_values($arr);
}

/** @var array<string, int> $m */
$m = ['a' => 1];
foreach (gen_only_values($m) as $v) {
    echo $v + 1;
}
