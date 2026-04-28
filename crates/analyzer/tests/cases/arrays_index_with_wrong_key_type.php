<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function read_with_int_key(array $arr): int
{
    // @mago-expect analysis:mismatched-array-index,possibly-undefined-int-array-index
    return $arr[5];
}
