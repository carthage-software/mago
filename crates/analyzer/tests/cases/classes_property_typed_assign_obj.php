<?php

declare(strict_types=1);

final class ClassesTypedObjA
{
}

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
    /** @mago-expect analysis:invalid-property-assignment-value */
    $obj->ref = new ClassesTypedObjB();
}
