<?php

declare(strict_types=1);

final class ClassesTooManyArgs
{
    public function take(int $a): int
    {
        return $a;
    }
}

function classesTooMany(): void
{
    (new ClassesTooManyArgs())->take(1, 2);
}
