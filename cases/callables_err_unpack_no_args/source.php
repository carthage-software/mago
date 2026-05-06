<?php

declare(strict_types=1);

function callables_needs_one_arg(int $n): int
{
    return $n;
}

callables_needs_one_arg(...[]);
