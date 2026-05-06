<?php

declare(strict_types=1);

final class ClassesCtorDefaultInit
{
    public string $name;

    public function __construct(string $name = 'default')
    {
        $this->name = $name;
    }
}

echo (new ClassesCtorDefaultInit())->name;
