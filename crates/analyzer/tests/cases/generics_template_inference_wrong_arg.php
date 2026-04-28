<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param list<T> $list
 *
 * @return T|null
 */
function gen_first2(array $list): mixed
{
    return $list[0] ?? null;
}

function takes_nullable_int_2(?int $n): void
{
}

/** @mago-expect analysis:possibly-invalid-argument */
takes_nullable_int_2(gen_first2(['a', 'b']));
