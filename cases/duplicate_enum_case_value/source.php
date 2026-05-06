<?php

declare(strict_types=1);

enum StringEnum: string
{
    case One = 'one';
    case Two = 'one';
    case Three = 'three';
}

enum IntEnum: int
{
    case Active = 1;
    case Inactive = 2;
    case Disabled = 1;
}

enum NoDuplicates: string
{
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}
