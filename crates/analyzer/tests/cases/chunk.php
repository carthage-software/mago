<?php

/**
 * @template T
 *
 * @param iterable<T> $iterable
 * @param positive-int $size
 *
 * @return list<list<T>>
 */
function chunk(iterable $iterable, int $size): array
{
    $result = [];
    $ii = 0;
    $chunk_number = -1;
    foreach ($iterable as $value) {
        if (($ii % $size) === 0) {
            $result[] = [];
            $chunk_number++;
        }

        $result[$chunk_number][] = $value;
        $ii++;
    }

    return array_values($result);
}
