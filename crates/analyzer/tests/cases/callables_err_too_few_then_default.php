<?php

declare(strict_types=1);

function callables_a_required_b_default(int $a, int $b = 0): int
{
    return $a + $b;
}

/** @mago-expect analysis:too-few-arguments */
callables_a_required_b_default();
