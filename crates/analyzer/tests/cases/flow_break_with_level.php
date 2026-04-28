<?php

declare(strict_types=1);

/**
 * @param list<list<int>> $matrix
 */
function flow_break_with_level(array $matrix, int $target): null|int
{
    foreach ($matrix as $row) {
        foreach ($row as $value) {
            if ($value === $target) {
                return $target;
            }
            if ($value < 0) {
                break 2;
            }
        }
    }

    return null;
}
