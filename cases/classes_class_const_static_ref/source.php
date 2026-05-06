<?php

declare(strict_types=1);

class ClassesConstStaticRef
{
    public const string TAG = 'base';

    public function tag(): string
    {
        return static::TAG;
    }
}

echo (new ClassesConstStaticRef())->tag();
