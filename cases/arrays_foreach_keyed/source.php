<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function describe_pairs(array $arr): string
{
    $out = '';
    foreach ($arr as $k => $v) {
        $out .= $k . '=' . (string) $v;
    }
    return $out;
}
