<?php

declare(strict_types=1);

final class ClassesConstSelfRef
{
    public const string GREETING = 'hi';

    public function greet(): string
    {
        return self::GREETING;
    }
}

echo (new ClassesConstSelfRef())->greet();
