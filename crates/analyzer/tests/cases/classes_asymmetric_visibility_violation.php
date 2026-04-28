<?php

declare(strict_types=1);

final class ClassesAsymViolation
{
    public private(set) int $value = 0;
}

function classesAsymViol(ClassesAsymViolation $obj): void
{
    /** @mago-expect analysis:invalid-property-write */
    $obj->value = 5;
}
