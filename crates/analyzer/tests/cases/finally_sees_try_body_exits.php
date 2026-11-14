<?php

declare(strict_types=1);

function finally_sees_early_return(): int
{
    $x = null;
    try {
        $x = 1;
        if (rand(0, 1) === 1) {
            return 5;
        }
        $x = 'a';
    } finally {
        if ($x === 1) {
            echo 'returned early';
        }
    }

    return 0;
}

function finally_sees_work(): void {}

/** @throws Throwable */
function finally_sees_rethrow(): void
{
    $retry = false;
    try {
        try {
            finally_sees_work();
        } catch (Throwable $e) {
            $retry = true;
            throw $e;
        }
    } finally {
        if (!$retry) {
            echo 'no retry';
        }
    }
}
