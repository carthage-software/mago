<?php

declare(strict_types=1);

function flow_for_loop_init_var(int $start, int $end): int
{
    $sum = 0;
    for ($i = $start; $i < $end; $i++) {
        $sum += $i;
    }

    return $sum;
}
