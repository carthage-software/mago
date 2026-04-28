<?php

declare(strict_types=1);

/**
 * @param list{int, int} $pair
 */
function sum_pair(array $pair): int
{
    list($a, $b) = $pair;
    return $a + $b;
}
