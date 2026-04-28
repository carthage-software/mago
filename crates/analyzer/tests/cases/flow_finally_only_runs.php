<?php

declare(strict_types=1);

function flow_finally_only_runs(int $v): int
{
    $r = 0;

    try {
        if ($v > 0) {
            $r = 1;
        }
    } finally {
        $r += 100;
    }

    return $r;
}
