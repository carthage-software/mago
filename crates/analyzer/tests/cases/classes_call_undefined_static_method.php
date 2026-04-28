<?php

declare(strict_types=1);

final class ClassesUndefStaticMethod
{
    public static function defined(): int
    {
        return 1;
    }
}

function classesUndefStaticMethod(): void
{
    /** @mago-expect analysis:non-existent-method */
    ClassesUndefStaticMethod::bogus();
}
