<?php

declare(strict_types=1);

final class Math
{
    public static function dbl(int $n): int
    {
        return $n * 2;
    }
}

Math::dbl('not int');
