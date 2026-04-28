<?php

declare(strict_types=1);

function flow_for_loop_iteration_var(int $n): int
{
    $last = -1;
    for ($i = 0; $i < $n; $i++) {
        $last = $i;
    }

    return $last;
}
