<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: string} $arr
 */
function bad_access(array $arr): mixed
{
    // @mago-expect analysis:undefined-string-array-index
    return $arr['c'];
}
