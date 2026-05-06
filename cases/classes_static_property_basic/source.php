<?php

declare(strict_types=1);

final class ClassesStaticPropertyBasic
{
    public static int $counter = 0;

    public static function bump(): void
    {
        self::$counter++;
    }
}

ClassesStaticPropertyBasic::bump();
echo ClassesStaticPropertyBasic::$counter;
