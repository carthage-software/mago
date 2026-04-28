<?php

declare(strict_types=1);

function callables_default_required(int $a, int $b = 10): int
{
    return $a + $b;
}

/** @mago-expect analysis:too-few-arguments */
callables_default_required();
