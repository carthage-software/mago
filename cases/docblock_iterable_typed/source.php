<?php

declare(strict_types=1);

/**
 * @param iterable<int, string> $it
 */
function joinItemsCH(iterable $it): string
{
    $out = '';
    foreach ($it as $v) {
        $out .= $v;
    }

    return $out;
}

echo joinItemsCH(['a', 'b']);
