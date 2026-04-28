<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return int<0, max>
 */
function sz(array $arr): int
{
    return sizeof($arr);
}
