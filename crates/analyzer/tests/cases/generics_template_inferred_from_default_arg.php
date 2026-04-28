<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param array<int, T> $arr
 * @param T $default
 *
 * @return T
 */
function gen_get_or_default(array $arr, int $key, mixed $default): mixed
{
    return $arr[$key] ?? $default;
}

function take_int_only(int $n): void
{
}

take_int_only(gen_get_or_default([1, 2, 3], 0, 99));
