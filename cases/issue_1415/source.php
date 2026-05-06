<?php

declare(strict_types=1);

interface Iface1415
{
    public array $myProperty { get; }
}

trait TraitWithProperty1415
{
    public array $myProperty = [[1, 2, 3]];
}

class MyClass1415 implements Iface1415
{
    use TraitWithProperty1415;
}
