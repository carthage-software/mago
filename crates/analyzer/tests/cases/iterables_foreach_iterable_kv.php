<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

function take_int(int $_n): void
{
}

/**
 * @param iterable<string, int> $it
 */
function iterate_kv(iterable $it): void
{
    foreach ($it as $k => $v) {
        take_string($k);
        take_int($v);
    }
}
