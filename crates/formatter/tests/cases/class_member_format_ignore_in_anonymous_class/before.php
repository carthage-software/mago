<?php

$obj = new class {
    // @mago-format-ignore-next
    public const   A   =   1;

    public const B = 2;

    // @mago-format-ignore-start
    public    $x   =   1;
    public    $y   =   2;
    // @mago-format-ignore-end

    public $z = 3;

    // @mago-format-ignore-next
    public function   foo()   {   return 1;   }

    public function bar(): int
    {
        return 2;
    }
};
