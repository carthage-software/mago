<?php

declare(strict_types=1);

final class ClassesCloneUnionInt
{
}

function classesCloneUnion(ClassesCloneUnionInt|int $value): mixed
{
    /** @mago-expect analysis:possibly-invalid-clone */
    return clone $value;
}
