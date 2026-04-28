<?php

declare(strict_types=1);

final class ClassesMethodObjA
{
}

final class ClassesMethodObjB
{
    public function take(ClassesMethodObjA $a): void
    {
        unset($a);
    }
}

function classesMethodObjBad(): void
{
    /** @mago-expect analysis:invalid-argument */
    (new ClassesMethodObjB())->take(new ClassesMethodObjB());
}
