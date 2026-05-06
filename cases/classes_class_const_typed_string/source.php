<?php

declare(strict_types=1);

final class ClassesTypedConstString
{
    public const string NAME = 'mago';
    public const int VERSION = 1;
    public const array LIST = [1, 2];
}

echo ClassesTypedConstString::NAME;
echo ClassesTypedConstString::VERSION;
echo count(ClassesTypedConstString::LIST);
