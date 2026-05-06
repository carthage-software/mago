<?php

declare(strict_types=1);

final class Limits
{
    public const int DEFAULT_MAX = 100;
}

function callables_clamp(int $value, int $max = Limits::DEFAULT_MAX): int
{
    return $value > $max ? $max : $value;
}

echo callables_clamp(5);
echo callables_clamp(5, 200);
