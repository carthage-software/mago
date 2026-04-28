<?php

declare(strict_types=1);

class ClassesLSBBase
{
    public static function name(): string
    {
        return static::class;
    }
}

final class ClassesLSBChild extends ClassesLSBBase
{
}

echo ClassesLSBChild::name();
