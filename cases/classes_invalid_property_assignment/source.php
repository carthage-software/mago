<?php

declare(strict_types=1);

final class ClassesInvalidPropAsgn
{
    public int $count = 0;
}

function classesAssignWrongType(ClassesInvalidPropAsgn $obj): void
{
    $obj->count = 'string';
}
