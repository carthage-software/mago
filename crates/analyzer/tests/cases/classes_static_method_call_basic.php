<?php

declare(strict_types=1);

final class ClassesStaticCall
{
    public static function get(): int
    {
        return 1;
    }
}

echo ClassesStaticCall::get();
