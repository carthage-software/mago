<?php

declare(strict_types=1);

function flow_while_loop_narrow(int $n): int
{
    $result = 0;

    while ($n > 0) {
        $result += $n;
        $n--;
    }

    return $result;
}
