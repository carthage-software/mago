<?php

declare(strict_types=1);

/**
 * @param array{a?: int} $arr
 */
function with_default(array $arr): int
{
    return $arr['a'] ?? 0;
}
