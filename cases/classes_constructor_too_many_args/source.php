<?php

declare(strict_types=1);

final class ClassesCtorTooManyArgs
{
    public function __construct(
        public int $x = 0,
    ) {}
}

function classesCtorTooMany(): void
{
    $_ = new ClassesCtorTooManyArgs(1, 2);
}
