<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return array<string, int>
 */
function drop_maybe(array $arr): array
{
    unset($arr['no_such_key']);
    return $arr;
}
