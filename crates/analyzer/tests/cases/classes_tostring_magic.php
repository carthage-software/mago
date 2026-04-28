<?php

declare(strict_types=1);

final class ClassesToStringMagic
{
    public function __construct(private string $value)
    {
    }

    public function __toString(): string
    {
        return $this->value;
    }
}

echo (new ClassesToStringMagic('hi'))->__toString();
