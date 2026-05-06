<?php

declare(strict_types=1);

function callables_strict_int(int $n): int
{
    return $n;
}

callables_strict_int('5');
