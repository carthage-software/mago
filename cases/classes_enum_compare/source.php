<?php

declare(strict_types=1);

enum ClassesEnumCmp
{
    case A;
    case B;
}

function classesEnumCmpFn(ClassesEnumCmp $x): bool
{
    return $x === ClassesEnumCmp::A;
}

echo classesEnumCmpFn(ClassesEnumCmp::A) ? '1' : '0';
echo classesEnumCmpFn(ClassesEnumCmp::B) ? '1' : '0';
