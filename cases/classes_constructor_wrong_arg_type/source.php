<?php

declare(strict_types=1);

final class ClassesCtorWrongArg
{
    public function __construct(
        public int $x,
    ) {}
}

function classesCtorWrongArg(): void
{
    $_ = new ClassesCtorWrongArg('not-int');
}
