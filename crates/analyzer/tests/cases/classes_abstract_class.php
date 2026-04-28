<?php

declare(strict_types=1);

abstract class ClassesAbstractBase
{
    public function name(): string
    {
        return static::class;
    }
}

final class ClassesAbstractChild extends ClassesAbstractBase
{
}

echo (new ClassesAbstractChild())->name();
