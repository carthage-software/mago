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
function gen_take_if(mixed $value, callable $pred): mixed
{
    return $pred($value) ? $value : null;
}

function takes_int_or_null_p(?int $n): void
{
}

takes_int_or_null_p(gen_take_if(5, static fn(int $n): bool => $n > 0));
