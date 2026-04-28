<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

function take_int(int $_n): void
{
}

/**
 * @param Iterator<string, int> $it
 */
function iterate_iter(Iterator $it): void
{
    foreach ($it as $key => $value) {
        take_string($key);
        take_int($value);
    }
}
