<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 * @param callable(T): T $fn
 *
 * @return T
 */
function gen_apply(mixed $value, callable $fn): mixed
{
    return $fn($value);
}

function takes_int_apply(int $n): void
{
}

takes_int_apply(gen_apply(1, static fn(int $n): int => $n + 1));
