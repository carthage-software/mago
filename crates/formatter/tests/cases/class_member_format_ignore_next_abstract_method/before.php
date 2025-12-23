<?php

abstract class Foo
{
    // @mago-format-ignore-next
    abstract public function   foo(   $a  ,   $b  )  ;

    abstract public function bar(): int;

    // @mago-format-ignore-next
    abstract protected function   baz()  :   string  ;
}
