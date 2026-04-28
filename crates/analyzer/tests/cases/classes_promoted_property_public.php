<?php

declare(strict_types=1);

final class ClassesPromotedPublic
{
    public function __construct(public int $value)
    {
    }
}

echo (new ClassesPromotedPublic(7))->value;
