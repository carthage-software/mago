<?php

declare(strict_types=1);

function takesInt(int $n): int
{
    return $n;
}

takesInt(PHP_INT_MAX);
takesInt(PHP_INT_MIN);
takesInt(PHP_INT_SIZE);
