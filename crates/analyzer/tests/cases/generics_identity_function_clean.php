<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 *
 * @return T
 */
function gen_identity(mixed $value): mixed
{
    return $value;
}

/**
 * @param int $n
 */
function take_int(int $n): void
{
}

take_int(gen_identity(42));
