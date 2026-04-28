<?php

declare(strict_types=1);

/**
 * @param callable(string): int $cb
 */
function callables_string_to_int_cb(callable $cb): int
{
    /** @mago-expect analysis:invalid-argument */
    return $cb(1);
}

callables_string_to_int_cb(fn(string $s): int => strlen($s));
