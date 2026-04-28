<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 * @param callable(T): bool $pred
 *
 * @return T|null
 */
function gen_take_if2(mixed $value, callable $pred): mixed
{
    return $pred($value) ? $value : null;
}

/** @mago-expect analysis:invalid-argument */
gen_take_if2(5, static fn(string $s): bool => '' !== $s);
