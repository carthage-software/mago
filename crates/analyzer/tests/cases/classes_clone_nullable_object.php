<?php

declare(strict_types=1);

final class ClassesCloneNullable
{
    public int $x = 0;
}

function classesCloneNullable(null|ClassesCloneNullable $obj): null|ClassesCloneNullable
{
    if (null === $obj) {
        return null;
    }

    return clone $obj;
}

$_ = classesCloneNullable(new ClassesCloneNullable());
$_ = classesCloneNullable(null);
