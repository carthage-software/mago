<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return list<int>
 */
function vals(array $arr): array
{
    return array_values($arr);
}
