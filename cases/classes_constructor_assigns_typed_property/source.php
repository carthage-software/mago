<?php

declare(strict_types=1);

final class ClassesCtorAssignsTyped
{
    public string $name;

    public function __construct(string $name)
    {
        $this->name = $name;
    }
}

echo (new ClassesCtorAssignsTyped('mago'))->name;
