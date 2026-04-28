<?php

declare(strict_types=1);

final class ClassesNewSelf
{
    public static function build(): self
    {
        return new self();
    }
}

$_ = ClassesNewSelf::build();
