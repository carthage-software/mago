<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return int<0, max>
 */
function size(array $arr): int
{
    return count($arr);
}
