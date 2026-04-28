<?php

declare(strict_types=1);

/** @mago-expect analysis:missing-return-statement */
function callables_returns_int_no_ret(int $n): int
{
    if ($n > 0) {
        return $n;
    }
}

echo callables_returns_int_no_ret(1);
