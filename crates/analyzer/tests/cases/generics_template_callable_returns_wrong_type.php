<?php

declare(strict_types=1);

/**
 * @template T
 * @template U
 *
 * @param T $value
 * @param callable(T): U $fn
 *
 * @return U
 */
function gen_apply_uw(mixed $value, callable $fn): mixed
{
    return $fn($value);
}

function takes_int_uw(int $n): void
{
}

/** @mago-expect analysis:invalid-argument */
takes_int_uw(gen_apply_uw('hi', static fn(string $s): string => $s));
