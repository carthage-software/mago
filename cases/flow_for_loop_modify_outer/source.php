<?php

declare(strict_types=1);

function flow_for_loop_modify_outer(): int
{
    $count = 0;
    for ($i = 1; $i <= 10; $i++) {
        $count = $i;
    }

    return $count;
}
