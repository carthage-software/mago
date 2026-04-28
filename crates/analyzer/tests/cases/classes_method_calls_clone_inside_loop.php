<?php

declare(strict_types=1);

final class ClassesCloneInLoop
{
    public int $value = 0;
}

function classesCloneLoop(): void
{
    $obj = new ClassesCloneInLoop();
    for ($i = 0; $i < 3; $i++) {
        $copy = clone $obj;
        $copy->value = $i;
    }
}

classesCloneLoop();
