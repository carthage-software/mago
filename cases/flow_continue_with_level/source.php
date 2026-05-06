<?php

declare(strict_types=1);

/**
 * @param list<list<int>> $matrix
 */
function flow_continue_with_level(array $matrix, int $threshold): int
{
    $count = 0;

    foreach ($matrix as $row) {
        foreach ($row as $value) {
            if ($value < $threshold) {
                continue 2;
            }
            $count++;
        }
    }

    return $count;
}
