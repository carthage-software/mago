<?php

class Outer
{
    public function outer(): object
    {
        return new class {
            // @mago-format-ignore-next
            public const   A   =   1;

            public function inner(): object
            {
                return new class {
                    // @mago-format-ignore-next
                    public const   B   =   2;

                    // @mago-format-ignore-start
                    public    $x   =   1;
                    public    $y   =   2;
                    // @mago-format-ignore-end
                };
            }
        };
    }
}
