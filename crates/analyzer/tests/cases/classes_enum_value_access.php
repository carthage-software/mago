<?php

declare(strict_types=1);

enum ClassesEnumValueAccess: string
{
    case A = 'a';
    case B = 'b';
}

$x = ClassesEnumValueAccess::A;
echo $x->value;
echo $x->name;
