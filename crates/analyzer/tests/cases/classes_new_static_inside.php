<?php

declare(strict_types=1);

final class ClassesNewStatic
{
    public static function build(): static
    {
        return new static();
    }
}

$_ = ClassesNewStatic::build();
