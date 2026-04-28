<?php

declare(strict_types=1);

final class ClassesInvalidPropAsgn
{
    public int $count = 0;
}

function classesAssignWrongType(ClassesInvalidPropAsgn $obj): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    $obj->count = 'string';
}
