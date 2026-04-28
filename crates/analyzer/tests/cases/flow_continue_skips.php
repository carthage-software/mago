<?php

declare(strict_types=1);

/**
 * @param list<int> $items
 */
function flow_continue_skips(array $items): int
{
    $sum = 0;

    foreach ($items as $value) {
        if ($value < 0) {
            continue;
        }

        $sum += $value;
    }

    return $sum;
}
