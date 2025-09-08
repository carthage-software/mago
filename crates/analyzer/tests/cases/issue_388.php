<?php

/**
 * @param list<int> $test
 */
function x(array $test): int
{
    if (count($test) !== 1) {
        return -1;
    }

    $item = end($test);

    return $item;
}
