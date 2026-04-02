<?php

declare(strict_types=1);

function test_require_then_check(string $dir, string $mod, string $rest): int
{
    $func = $mod . '_action_handler_' . $rest;
    if (function_exists($func)) {
        $fn = $func;

        return 1;
    }

    $bn = "test.action.php";
    require_once "{$dir}/{$bn}";
    if (function_exists($func)) {
        $fn = $func;

        return 1;
    }

    return 0;
}
