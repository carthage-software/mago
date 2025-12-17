<?php

class Foo
{
    public const A = 1;

    // @mago-format-ignore-start
    public const   B =    2;
    public const   C =    3;
    public    $prop =    123;
    // @mago-format-ignore-end

    public const D = 4;

    public function normal(): int
    {
        return 1;
    }
}
