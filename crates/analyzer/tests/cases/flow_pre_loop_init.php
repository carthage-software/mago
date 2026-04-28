<?php

declare(strict_types=1);

function flow_pre_loop_init(): int
{
    $i = 0;
    while ($i < 3) {
        $i++;
    }

    return $i;
}
