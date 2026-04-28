<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 *
 * @return T
 */
function gen_id_a(mixed $value): mixed
{
    return $value;
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return T
 */
function gen_id_b(mixed $value): mixed
{
    return gen_id_a($value);
}

function takes_int_2level(int $n): void
{
}

takes_int_2level(gen_id_b(42));
