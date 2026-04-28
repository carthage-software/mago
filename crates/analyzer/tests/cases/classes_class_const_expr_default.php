<?php

declare(strict_types=1);

final class ClassesConstExprDefault
{
    public const int A = 1;
    public const int B = self::A + 1;
}

echo ClassesConstExprDefault::B;
