<?php

declare(strict_types=1);

final class ClassesCtorWrongArg
{
    public function __construct(public int $x)
    {
    }
}

function classesCtorWrongArg(): void
{
    /** @mago-expect analysis:invalid-argument */
    $_ = new ClassesCtorWrongArg('not-int');
}
