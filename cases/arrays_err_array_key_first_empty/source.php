<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function bad(array $arr): string
{
    return array_key_first($arr);
}
