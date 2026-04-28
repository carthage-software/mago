<?php

declare(strict_types=1);

final class ClassesStaticBadAssign
{
    public static int $count = 0;
}

function classesStaticBadAssign(): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    ClassesStaticBadAssign::$count = 'string';
}
