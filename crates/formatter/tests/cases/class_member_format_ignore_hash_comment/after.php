<?php

class Foo
{
    # @mago-format-ignore-next
    public const   A   =   1;

    public const B = 2;

    # @mago-format-ignore-start
    public const   C   =   3;
    public const   D   =   4;
    # @mago-format-ignore-end

    public const E = 5;
}
