<?php

declare(strict_types=1);

/**
 * @param list<int> $items
 */
function flow_isset_after_loop(array $items): int
{
    $found = null;

    foreach ($items as $item) {
        if ($item > 10) {
            $found = $item;
            break;
        }
    }

    if ($found === null) {
        return -1;
    }

    return $found;
}
