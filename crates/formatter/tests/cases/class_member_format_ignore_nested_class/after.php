<?php

class Outer
{
    // @mago-format-ignore-next
    public const   OUTER_CONST   =   1;

    public function getInner(): object
    {
        return new class {
            // @mago-format-ignore-next
            public const   INNER_CONST   =   2;

            public const NORMAL = 3;

            // @mago-format-ignore-start
            public    $x   =   1;
            public    $y   =   2;
            // @mago-format-ignore-end
        };
    }
}
