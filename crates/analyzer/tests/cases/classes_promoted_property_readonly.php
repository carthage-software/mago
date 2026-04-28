<?php

declare(strict_types=1);

final class ClassesPromotedReadonly
{
    public function __construct(public readonly string $id)
    {
    }
}

echo (new ClassesPromotedReadonly('x'))->id;
