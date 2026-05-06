<?php

declare(strict_types=1);

/**
 * @template T of array-key
 *
 * @param array<T, T> $map
 *
 * @return list<T>
 */
function gen_keys_eq_values(array $map): array
{
    $out = [];
    foreach ($map as $k => $v) {
        if ($k === $v) {
            $out[] = $v;
        }
    }
    return $out;
}

/** @var array<int, int> $m */
$m = [1 => 1, 2 => 2];
gen_keys_eq_values($m);
