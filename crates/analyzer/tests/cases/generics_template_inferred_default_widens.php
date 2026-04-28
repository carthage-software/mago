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
function gen_get_or_default2(array $arr, int $key, mixed $default): mixed
{
    return $arr[$key] ?? $default;
}

function take_int_only2(int $n): void
{
}

/** @mago-expect analysis:possibly-invalid-argument */
take_int_only2(gen_get_or_default2([1, 2, 3], 0, 'fallback'));
