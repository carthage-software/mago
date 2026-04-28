<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param list<T> $list
 *
 * @return T|null
 */
function gen_first_FE(array $list): mixed
{
    return $list[0] ?? null;
}

function takes_string_only(string $s): void
{
}

/** @mago-expect analysis:invalid-argument,possibly-null-argument */
takes_string_only(gen_first_FE([1, 2, 3]));
