<?php

declare(strict_types=1);

final class ClassesReadonlyWriteOutside
{
    public function __construct(
        public readonly string $id,
    ) {}
}

function classesReadonlyOutside(): void
{
    $obj = new ClassesReadonlyWriteOutside('one');
    $obj->id = 'two';
}
