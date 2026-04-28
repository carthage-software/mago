<?php

declare(strict_types=1);

/** @param callable(int): int $cb */
function callables_typed_var_cb(callable $cb): int
{
    /** @mago-expect analysis:invalid-argument */
    return $cb('bad');
}

callables_typed_var_cb(fn(int $n): int => $n);
