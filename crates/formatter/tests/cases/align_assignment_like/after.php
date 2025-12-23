<?php

class Example
{
    public const FOO    = 1;
    public const BARRRR = 2;
    public const B      = 3;

    public string $a     = 'a';
    public string $bbbbb = 'b';
    public string $cc    = 'c';
}

enum Status: int
{
    case Pending  = 0;
    case Approved = 1;
    case Rejected = 2;
}
