<?php

trait MyTrait
{
    // @mago-format-ignore-next
    public    $prop   =   123;

    public $normal = 456;

    // @mago-format-ignore-start
    public function   foo()   {   return 1;   }
    public function   bar()   {   return 2;   }
    // @mago-format-ignore-end

    public function baz(): int
    {
        return 3;
    }
}
