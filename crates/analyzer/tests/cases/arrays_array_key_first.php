<?php

declare(strict_types=1);

/**
 * @param non-empty-array<string, int> $arr
 */
function first_key(array $arr): string
{
    return array_key_first($arr);
}
