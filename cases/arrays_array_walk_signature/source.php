<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function consume(array $arr): void
{
    array_walk($arr, static function (int $v, string $k): void {
        echo $k, '=', (string) $v;
    });
}
