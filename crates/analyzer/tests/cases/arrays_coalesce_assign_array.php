<?php

declare(strict_types=1);

/**
 * @param array{a?: int} $arr
 * @return array{a: int}
 */
function set_default(array $arr): array
{
    $arr['a'] ??= 0;
    return $arr;
}
