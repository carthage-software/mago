<?php

declare(strict_types=1);

final class ClassesPromOnly
{
    public function __construct(public int $a, public int $b)
    {
    }

    public function sum(): int
    {
        return $this->a + $this->b;
    }
}

echo (new ClassesPromOnly(1, 2))->sum();
