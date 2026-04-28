<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

function take_string(string $_s): void
{
}

/**
 * @param list<string> $items
 */
function iterate_list(array $items): void
{
    foreach ($items as $key => $value) {
        take_int($key);
        take_string($value);
    }
}
