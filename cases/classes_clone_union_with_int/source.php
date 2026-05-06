<?php

declare(strict_types=1);

final class ClassesCloneUnionInt {}

function classesCloneUnion(ClassesCloneUnionInt|int $value): mixed
{
    return clone $value;
}
