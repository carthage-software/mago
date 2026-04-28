<?php

declare(strict_types=1);

final class Math
{
    public static function dbl(int $n): int
    {
        return $n * 2;
    }
}

/** @mago-expect analysis:invalid-argument */
Math::dbl('not int');
