<?php

declare(strict_types=1);

final class ClassesProtectedPropOutsideAccess
{
    protected int $value = 0;

    public function get(): int
    {
        return $this->value;
    }
}

/**
 * @mago-expect analysis:invalid-property-read
 * @mago-expect analysis:never-return
 */
function classesProtOutside(ClassesProtectedPropOutsideAccess $obj): int
{
    return $obj->value;
}
