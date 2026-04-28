<?php

declare(strict_types=1);

function flow_for_loop_narrow(): int
{
    $sum = 0;

    for ($i = 0; $i < 10; $i++) {
        $sum += $i;
    }

    return $sum;
}
