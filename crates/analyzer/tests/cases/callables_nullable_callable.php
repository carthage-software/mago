<?php

declare(strict_types=1);

/**
 * @param null|callable(int): int $cb
 */
function callables_optional_cb(null|callable $cb, int $n): int
{
    if ($cb === null) {
        return $n;
    }
    return $cb($n);
}

echo callables_optional_cb(null, 5);
echo callables_optional_cb(fn(int $x): int => $x + 1, 5);
