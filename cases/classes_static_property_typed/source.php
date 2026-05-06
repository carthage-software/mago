<?php

declare(strict_types=1);

final class ClassesStaticTypedProp
{
    public static string $name = 'mago';
}

ClassesStaticTypedProp::$name = 'bonjour';
echo ClassesStaticTypedProp::$name;
