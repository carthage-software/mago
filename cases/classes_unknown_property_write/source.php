<?php

declare(strict_types=1);

final class ClassesUnknownPropWrite
{
    public int $foo = 0;
}

function classesUnknownPropWrite(ClassesUnknownPropWrite $obj): void
{
    $obj->bar = 1;
}
