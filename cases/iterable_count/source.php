<?php

/**
 * @template T
 *
 * @param iterable<T> $iterable
 *
 * @return int<0, max>
 */
function count_elements(iterable $iterable): int
{
    if (is_countable($iterable)) {
        return count($iterable);
    }

    $count = 0;
    foreach ($iterable as $_) {
        ++$count;
    }

    return $count;
}
