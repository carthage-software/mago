<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: string} $arr
 */
function bad_access(array $arr): mixed
{
    return $arr['c'];
}
