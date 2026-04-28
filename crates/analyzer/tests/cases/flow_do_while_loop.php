<?php

declare(strict_types=1);

function flow_do_while_loop(int $n): int
{
    $i = 0;

    do {
        $i++;
    } while ($i < $n);

    return $i;
}
