<?php

declare(strict_types=1);

final class ClassesPromotedPrivate
{
    public function __construct(private int $value)
    {
    }

    public function get(): int
    {
        return $this->value;
    }
}

echo (new ClassesPromotedPrivate(7))->get();
