<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return array<string, int>
 */
function drop_x(array $arr): array
{
    unset($arr['x']);
    return $arr;
}
