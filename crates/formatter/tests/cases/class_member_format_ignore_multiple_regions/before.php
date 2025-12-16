<?php

class Foo
{
    // @mago-format-ignore-start
    public const   A   =   1;
    public const   B   =   2;
    // @mago-format-ignore-end

    public const C = 3;

    // @mago-format-ignore-start
    public    $prop1   =   1;
    public    $prop2   =   2;
    // @mago-format-ignore-end

    public $prop3 = 3;

    // @mago-format-ignore-start
    public function   foo()   {   return 1;   }
    public function   bar()   {   return 2;   }
    // @mago-format-ignore-end

    public function baz(): int
    {
        return 3;
    }
}
