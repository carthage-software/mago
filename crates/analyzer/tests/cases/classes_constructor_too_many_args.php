<?php

declare(strict_types=1);

final class ClassesCtorTooManyArgs
{
    public function __construct(public int $x = 0)
    {
    }
}

function classesCtorTooMany(): void
{
    /** @mago-expect analysis:too-many-arguments */
    $_ = new ClassesCtorTooManyArgs(1, 2);
}
