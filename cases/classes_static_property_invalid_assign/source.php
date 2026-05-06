<?php

declare(strict_types=1);

final class ClassesStaticBadAssign
{
    public static int $count = 0;
}

function classesStaticBadAssign(): void
{
    ClassesStaticBadAssign::$count = 'string';
}
