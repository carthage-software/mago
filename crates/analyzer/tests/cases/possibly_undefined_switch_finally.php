<?php

declare(strict_types=1);

namespace App;

use Closure;

function non_exhaustive_switch(string $mode): string
{
    switch ($mode) {
        case "a":
            $result = "apple";
            break;
        case "b":
            $result = "banana";
            break;
    }

    return $result; // @mago-expect analysis:possibly-undefined-variable
}

function exhaustive_switch_with_default(string $mode): string
{
    switch ($mode) {
        case "a":
            $result = "apple";
            break;
        default:
            $result = "other";
            break;
    }

    return $result;
}

/**
 * @param Closure(): string $work
 */
function try_finally_unassigned_before(Closure $work): void
{
    try {
        $msg = $work();
    } catch (\Throwable $e) {
        $msg = "failed";
    } finally {
        echo $msg; // @mago-expect analysis:possibly-undefined-variable
    }
}

/**
 * @param Closure(): string $work
 */
function try_finally_assigned_before(Closure $work): void
{
    $msg = "initial";
    try {
        $msg = $work();
    } finally {
        echo $msg;
    }
}
