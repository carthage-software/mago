<?php

declare(strict_types=1);

readonly final class ClassesReadonlyDecl
{
    public function __construct(public string $name)
    {
    }
}

echo (new ClassesReadonlyDecl('mago'))->name;
