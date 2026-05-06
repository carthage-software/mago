<?php

/**
 * @template Tk as array-key
 * @template Tv
 *
 * @param iterable<Tk, Tv> $iterable The iterable to chunk
 * @param positive-int $size The size of each chunk
 *
 * @return list<array<Tk, Tv>>
 */
function chunk_with_keys(iterable $iterable, int $size): array
{
    $result = [];
    $ii = 0;
    $chunk_number = -1;
    foreach ($iterable as $k => $value) {
        if (($ii % $size) === 0) {
            $chunk_number++;
            $result[$chunk_number] = [];
        }

        $result[$chunk_number][$k] = $value;
        $ii++;
    }

    return array_values($result);
}
