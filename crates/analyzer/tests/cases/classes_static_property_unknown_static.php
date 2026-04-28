<?php

declare(strict_types=1);

final class ClassesUndefStaticProp
{
    public static int $known = 0;
}

function classesUndefStaticProp(): void
{
    /** @mago-expect analysis:non-existent-property */
    ClassesUndefStaticProp::$bogus = 1;
}
