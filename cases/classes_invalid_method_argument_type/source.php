<?php

declare(strict_types=1);

final class ClassesMethodArgType
{
    public function take(int $a): int
    {
        return $a;
    }
}

function classesMethodArgType(): void
{
    (new ClassesMethodArgType())->take('str');
}
