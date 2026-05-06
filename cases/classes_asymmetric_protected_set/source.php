<?php

declare(strict_types=1);

class ClassesAsymProtSet
{
    public protected(set) int $value = 0;

    public function bump(): void
    {
        $this->value++;
    }
}

function classesAsymProtViol(ClassesAsymProtSet $obj): void
{
    $obj->value = 5;
}
