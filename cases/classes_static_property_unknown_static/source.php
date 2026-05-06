<?php

declare(strict_types=1);

final class ClassesUndefStaticProp
{
    public static int $known = 0;
}

function classesUndefStaticProp(): void
{
    ClassesUndefStaticProp::$bogus = 1;
}
