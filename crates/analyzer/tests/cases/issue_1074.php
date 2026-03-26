<?php

/**
 * @return 1
 */
function y1(): int
{
    $foo = [1 => 2, 17 => 1, 12 => 24];
    $keys = array_keys($foo);
    $min = min($keys);

    return $min;
}

/**
 * @return 17
 */
function y2(): int
{
    $foo = [1 => 2, 17 => 1, 12 => 24];
    $keys = array_keys($foo);
    $max = max($keys);

    return $max;
}
