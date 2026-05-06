<?php

declare(strict_types=1);

final class ClassesNonExistProp
{
    public int $x = 0;
}

function classesNonExistProp(ClassesNonExistProp $obj): mixed
{
    return $obj->y;
}
