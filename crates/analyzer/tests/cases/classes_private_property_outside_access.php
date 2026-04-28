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
 * @mago-expect analysis:invalid-property-read
 * @mago-expect analysis:never-return
 */
function classesPrivOutside(ClassesPrivatePropOutsideAccess $obj): int
{
    return $obj->secret;
}
