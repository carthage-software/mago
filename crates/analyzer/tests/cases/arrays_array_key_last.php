<?php

declare(strict_types=1);

/**
 * @param non-empty-array<string, int> $arr
 */
function last_key(array $arr): string
{
    return array_key_last($arr);
}
