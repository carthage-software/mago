<?php

declare(strict_types=1);

final class ClassesNonExistProp
{
    public int $x = 0;
}

function classesNonExistProp(ClassesNonExistProp $obj): mixed
{
    /** @mago-expect analysis:non-existent-property */
    return $obj->y;
}
