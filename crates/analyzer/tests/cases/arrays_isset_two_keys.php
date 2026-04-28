<?php

declare(strict_types=1);

/**
 * @param array{a?: int, b?: int} $arr
 */
function sum_present(array $arr): int
{
    if (isset($arr['a'], $arr['b'])) {
        return $arr['a'] + $arr['b'];
    }
    return 0;
}
