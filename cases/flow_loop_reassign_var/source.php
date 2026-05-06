<?php

declare(strict_types=1);

function flow_loop_reassign_var(int $bound): string
{
    $value = 'start';

    for ($i = 0; $i < $bound; $i++) {
        $value = (string) $i;
    }

    return $value;
}
