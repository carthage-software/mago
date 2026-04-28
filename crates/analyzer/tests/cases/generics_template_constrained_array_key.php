<?php

declare(strict_types=1);

/**
 * @template K of array-key
 *
 * @param array<K, mixed> $arr
 *
 * @return list<K>
 */
function gen_keys_only(array $arr): array
{
    return array_keys($arr);
}

/** @var array<int, string> $a */
$a = [1 => 'x', 2 => 'y'];
foreach (gen_keys_only($a) as $k) {
    echo $k + 1;
}
