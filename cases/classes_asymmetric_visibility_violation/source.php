<?php

declare(strict_types=1);

final class ClassesAsymViolation
{
    public private(set) int $value = 0;
}

function classesAsymViol(ClassesAsymViolation $obj): void
{
    $obj->value = 5;
}
