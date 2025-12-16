<?php

class Foo
{
    public const A = 1;

    // @mago-format-ignore-next
    public const GRID = [
      [1, 2, 3], [1, 2, ], [0,    0],
    ];

    public const B = 2;

    // @mago-format-ignore-next
    public    $prop =    123;

    public $normal = 456;

    // @mago-format-ignore-next
    public function   ignored()    {   return 1;   }

    public function normal(): int
    {
        return 2;
    }
}
