<?php

declare(strict_types=1);

final class ClassesInstanceofDyn
{
}

function classesInstanceofDyn(object $obj, ClassesInstanceofDyn $target): bool
{
    return $obj instanceof $target;
}

echo classesInstanceofDyn(new ClassesInstanceofDyn(), new ClassesInstanceofDyn()) ? 'y' : 'n';
