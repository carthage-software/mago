<?php

declare(strict_types=1);

function takesInt(int $n): int
{
    return $n;
}

takesInt(0);
takesInt(1);
takesInt(-1);
takesInt(42);
takesInt(PHP_INT_MAX);
takesInt(PHP_INT_MIN);
