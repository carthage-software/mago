<?php

declare(strict_types=1);

/**
 * @param array{a: int, b?: string} $arr
 */
function read_b_after_isset(array $arr): string
{
    if (isset($arr['b'])) {
        return $arr['b'];
    }
    return 'default';
}
