<?php

class Foo
{
    public const BEFORE = 1;

    // @mago-format-ignore-start
    public const   A   =   1;
    public const   B   =   2;
    public    $prop   =   123;
    public function   foo()   {   return 1;   }
    // @mago-format-ignore-end

    public const AFTER = 3;

    public function bar(): int
    {
        return 2;
    }
}
