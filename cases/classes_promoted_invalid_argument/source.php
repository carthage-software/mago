<?php

declare(strict_types=1);

final class ClassesPromInvArg
{
    public function __construct(
        public int $value,
    ) {}
}

function classesPromInvArg(): void
{
    $_ = new ClassesPromInvArg('not-int');
}
