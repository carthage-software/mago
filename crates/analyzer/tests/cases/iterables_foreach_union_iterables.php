<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

/**
 * @param list<string>|Iterator<int, string> $items
 */
function iterate(array|Iterator $items): void
{
    foreach ($items as $v) {
        take_string($v);
    }
}
