<?php

class Foo
{
    // @mago-format-ignore-next
    final public function   finalMethod()   {   return 1;   }

    final public function normalFinal(): int
    {
        return 2;
    }

    // @mago-format-ignore-next
    final protected static function   finalStaticMethod()   {   return 3;   }
}
