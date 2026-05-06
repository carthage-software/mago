<?php

declare(strict_types=1);

final class Util
{
    public static function only(int $n): int
    {
        return $n;
    }
}

Util::only(1, 2);
