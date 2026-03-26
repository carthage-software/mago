<?php

trait SomeTrait
{
    public function foo(): void {}
}

class Foo
{
    // @mago-format-ignore-next
    use    SomeTrait  {   foo as   bar ;   }

    public function baz(): void {}
}
