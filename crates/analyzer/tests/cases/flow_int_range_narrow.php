<?php

declare(strict_types=1);

/**
 * @param int<0, 100> $value
 */
function flow_int_range_narrow(int $value): int
{
    if ($value < 50) {
        return 1;
    }

    return 2;
}
