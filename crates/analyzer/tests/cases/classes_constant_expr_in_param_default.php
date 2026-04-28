<?php

declare(strict_types=1);

final class ClassesParamDefault
{
    public const int OFFSET = 5;

    public function build(int $base = self::OFFSET + 1): int
    {
        return $base;
    }
}

echo (new ClassesParamDefault())->build();
