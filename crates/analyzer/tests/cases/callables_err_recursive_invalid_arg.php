<?php

declare(strict_types=1);

function callables_int_recur(int $n): int
{
    if ($n <= 0) {
        return 0;
    }
    /** @mago-expect analysis:invalid-argument */
    return callables_int_recur('bad');
}

echo callables_int_recur(5);
