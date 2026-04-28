<?php

declare(strict_types=1);

/**
 * @param non-empty-array<string, int> $arr
 * @return int<1, max>
 */
function size(array $arr): int
{
    return count($arr);
}
