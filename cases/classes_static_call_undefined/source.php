<?php

declare(strict_types=1);

final class ClassesUndefStatic
{
    public static function known(): int
    {
        return 1;
    }
}

function classesUndefStatic(): void
{
    ClassesUndefStatic::unknown();
}
