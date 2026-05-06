<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return array<int, string>
 */
function flip(array $arr): array
{
    return array_flip($arr);
}
