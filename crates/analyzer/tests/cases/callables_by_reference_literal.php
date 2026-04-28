<?php

declare(strict_types=1);

function callables_set_to_one(int &$n): void
{
    $n = 1;
}

/** @mago-expect analysis:invalid-pass-by-reference */
callables_set_to_one(42);
