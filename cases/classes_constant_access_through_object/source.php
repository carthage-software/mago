<?php

declare(strict_types=1);

final class ClassesConstThroughObj
{
    public const string LABEL = 'mago';
}

$obj = new ClassesConstThroughObj();
echo $obj::LABEL;
