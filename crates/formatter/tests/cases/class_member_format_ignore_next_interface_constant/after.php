<?php

interface Foo
{
    // @mago-format-ignore-next
    public const   A   =   1;

    // @mago-format-ignore-next
    public const   B   =   [
      'key1'  =>  'value1',
      'key2'  =>  'value2',
    ];

    public const C = 3;

    // @mago-format-ignore-start
    public const   D   =   4;
    public const   E   =   5;
    // @mago-format-ignore-end
}
