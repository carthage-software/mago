<?php

declare(strict_types=1);

final class ClassesInvalidArg
{
    public function take(int $value): void
    {
        unset($value);
    }
}

function classesCallWithStr(): void
{
    (new ClassesInvalidArg())->take('not-an-int');
}
