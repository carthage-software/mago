<?php

declare(strict_types=1);

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $arr
 *
 * @return list<K>
 */
function gen_only_keys(array $arr): array
{
    return array_keys($arr);
}

/** @var array<string, int> $m */
$m = ['a' => 1];
foreach (gen_only_keys($m) as $k) {
    echo $k;
}
