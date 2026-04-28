<?php

declare(strict_types=1);

/**
 * @param array<int, int> $arr
 * @return list<int>
 */
function narrow(array $arr): array
{
    // @mago-expect analysis:invalid-return-statement
    return $arr;
}
