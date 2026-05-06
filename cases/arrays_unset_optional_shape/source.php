<?php

declare(strict_types=1);

/**
 * @param array{a: int, b?: string} $arr
 * @return array{a: int}
 */
function drop_b(array $arr): array
{
    unset($arr['b']);
    return $arr;
}
