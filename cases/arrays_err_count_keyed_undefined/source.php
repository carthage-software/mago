<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function bad(array $arr): int
{
    return $arr['unknown_key'];
}
