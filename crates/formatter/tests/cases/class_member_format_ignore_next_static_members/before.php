<?php

class Foo
{
    // @mago-format-ignore-next
    public static    $staticProp   =   123;

    public static $normalStaticProp = 456;

    // @mago-format-ignore-next
    public static function   staticMethod()   {   return 1;   }

    public static function normalStaticMethod(): int
    {
        return 2;
    }

    // @mago-format-ignore-next
    private static    $privateStatic   =   'test';
}
