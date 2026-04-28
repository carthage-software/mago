<?php

declare(strict_types=1);

final class ClassesPromInvArg
{
    public function __construct(public int $value)
    {
    }
}

function classesPromInvArg(): void
{
    /** @mago-expect analysis:invalid-argument */
    $_ = new ClassesPromInvArg('not-int');
}
