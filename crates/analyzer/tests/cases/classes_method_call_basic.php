<?php

declare(strict_types=1);

final class ClassesMethodCallBasic
{
    public function add(int $a, int $b): int
    {
        return $a + $b;
    }
}

echo (new ClassesMethodCallBasic())->add(1, 2);
