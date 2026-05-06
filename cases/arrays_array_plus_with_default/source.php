<?php

declare(strict_types=1);

/**
 * @param array{a: int, b?: string} $arr
 */
function destruct_with_default(array $arr): string
{
    ['a' => $a, 'b' => $b] = $arr + ['b' => 'default'];

    return $b;
}
