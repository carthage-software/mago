<?php

declare(strict_types=1);

function callables_needs_one_arg(int $n): int
{
    return $n;
}

/** @mago-expect analysis:too-few-arguments */
callables_needs_one_arg(...[]);
