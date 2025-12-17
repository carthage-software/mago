<?php

class Foo
{
    // @mago-format-ignore-next
    public readonly    string    $readonlyProp;

    public readonly int $normalReadonly;

    // @mago-format-ignore-next
    private readonly    array    $privateReadonly   =   [];
}
