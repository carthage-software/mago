<?php

declare(strict_types=1);

final class ClassesReturnSelf
{
    public static function build(): self
    {
        return new self();
    }
}

$_ = ClassesReturnSelf::build();
