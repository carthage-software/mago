<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param list<T> $list
 *
 * @return T|null
 */
function gen_first(array $list): mixed
{
    return $list[0] ?? null;
}

function take_nullable_int(?int $n): void
{
}

take_nullable_int(gen_first([1, 2, 3]));
