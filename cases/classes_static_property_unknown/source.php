<?php

declare(strict_types=1);

final class ClassesStaticUnknown
{
    public static int $known = 0;
}

function classesStaticUnknown(): mixed
{
    return ClassesStaticUnknown::$bogus;
}
