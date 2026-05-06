<?php

declare(strict_types=1);

final class ClassesPrivatePropOutsideAccess
{
    private int $secret = 0;

    public function get(): int
    {
        return $this->secret;
    }
}

/**
 */
function classesPrivOutside(ClassesPrivatePropOutsideAccess $obj): int
{
    return $obj->secret;
}
