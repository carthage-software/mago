<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function bad(array $arr): int
{
    // @mago-expect analysis:possibly-undefined-string-array-index
    return $arr['unknown_key'];
}
