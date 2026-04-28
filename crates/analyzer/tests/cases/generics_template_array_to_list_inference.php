<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param list<T> $list
 *
 * @return T|null
 */
function gen_last(array $list): mixed
{
    if ([] === $list) {
        return null;
    }
    return $list[count($list) - 1];
}

function takes_nullable_string(?string $s): void
{
}

takes_nullable_string(gen_last(['a', 'b']));
