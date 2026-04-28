<?php

declare(strict_types=1);

final class ClassesStaticMethodBasic
{
    public static function greet(string $who): string
    {
        return 'hi ' . $who;
    }
}

echo ClassesStaticMethodBasic::greet('mago');
