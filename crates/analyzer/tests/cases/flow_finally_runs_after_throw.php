<?php

declare(strict_types=1);

function flow_finally_runs_after_throw(): int
{
    $cleaned = false;

    try {
        try {
            throw new \RuntimeException('a');
        } finally {
            $cleaned = true;
        }
    } catch (\Throwable) {
    }

    return $cleaned ? 1 : 0;
}
