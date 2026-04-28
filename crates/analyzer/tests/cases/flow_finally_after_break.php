<?php

declare(strict_types=1);

function flow_finally_after_break(): int
{
    $i = 0;

    while (true) {
        try {
            $i++;
            break;
        } finally {
            $i += 10;
        }
    }

    return $i;
}
