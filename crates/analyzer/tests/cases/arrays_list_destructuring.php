<?php

declare(strict_types=1);

/**
 * @param list{int, string} $pair
 */
function take_pair(array $pair): string
{
    [$a, $b] = $pair;
    return $a . $b;
}
