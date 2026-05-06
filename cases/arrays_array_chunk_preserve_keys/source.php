<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return list<array<string, int>>
 */
function chunked(array $arr): array
{
    return array_chunk($arr, 2, true);
}
