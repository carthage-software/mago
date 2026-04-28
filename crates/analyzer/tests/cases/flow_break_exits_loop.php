<?php

declare(strict_types=1);

/**
 * @param list<int> $items
 */
function flow_break_exits_loop(array $items, int $target): int
{
    $found = -1;

    foreach ($items as $i => $value) {
        if ($value === $target) {
            $found = $i;
            break;
        }
    }

    return $found;
}
