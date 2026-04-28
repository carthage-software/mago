<?php

declare(strict_types=1);

final class ClassesCtorWiden
{
    public function __construct(public int|string $value)
    {
    }
}

echo (new ClassesCtorWiden(42))->value;
echo (new ClassesCtorWiden('mago'))->value;
