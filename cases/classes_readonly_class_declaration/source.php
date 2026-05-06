<?php

declare(strict_types=1);

final readonly class ClassesReadonlyDecl
{
    public function __construct(
        public string $name,
    ) {}
}

echo (new ClassesReadonlyDecl('mago'))->name;
