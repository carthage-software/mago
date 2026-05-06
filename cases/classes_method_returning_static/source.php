<?php

declare(strict_types=1);

/**
 * @consistent-constructor
 */
class ClassesReturnStatic
{
    public static function build(): static
    {
        return new static();
    }
}

$_ = ClassesReturnStatic::build();
