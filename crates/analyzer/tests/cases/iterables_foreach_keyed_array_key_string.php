<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

function take_int(int $_n): void
{
}

/**
 * @param array<string, int> $items
 */
function iterate_keyed(array $items): void
{
    foreach ($items as $key => $value) {
        take_string($key);
        take_int($value);
    }
}
