<?php

declare(strict_types=1);

final class ClassesArgClone
{
    public function take(int $a): void
    {
        unset($a);
    }
}

function classesArgClone(): void
{
    $obj = new ClassesArgClone();
    /** @mago-expect analysis:invalid-argument */
    $obj->take(clone $obj);
}
