<?php

class Foo
{
    // @mago-format-ignore-next
    #[Attribute]    #[Another(   1,   2   )]
    public    $prop =    123;

    #[Normal]
    public $normal = 456;

    // @mago-format-ignore-next
    #[Route(   '/foo'  ,    methods:   ['GET',   'POST']   )]
    public function   foo()    {   }
}
