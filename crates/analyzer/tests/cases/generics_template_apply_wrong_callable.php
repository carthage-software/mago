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
function gen_apply2(mixed $value, callable $fn): mixed
{
    return $fn($value);
}

/** @mago-expect analysis:invalid-argument */
gen_apply2(1, static fn(string $s): string => $s);
