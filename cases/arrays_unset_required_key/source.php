<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: string} $arr
 */
function bad(array $arr): array
{
    unset($arr['a']);
    return $arr;
}
