<?php

declare(strict_types=1);

/**
 * @param list<string> $items
 */
function flow_foreach_loop_narrow(array $items): int
{
    $count = 0;

    foreach ($items as $item) {
        $count += strlen($item);
    }

    return $count;
}
