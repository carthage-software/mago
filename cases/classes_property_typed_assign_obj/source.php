<?php

declare(strict_types=1);

final class ClassesTypedObjA {}

final class ClassesTypedObjB
{
    public ClassesTypedObjA $ref;

    public function __construct()
    {
        $this->ref = new ClassesTypedObjA();
    }
}

function classesTypedObjAssign(ClassesTypedObjB $obj): void
{
    $obj->ref = new ClassesTypedObjB();
}
