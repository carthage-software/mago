<?php

declare(strict_types=1);

final class ClassesDynamicProp
{
    public int $foo = 1;
    public int $bar = 2;
}

function classesDynPropRead(ClassesDynamicProp $obj, string $name): mixed
{
    return $obj->{$name};
}

$_ = classesDynPropRead(new ClassesDynamicProp(), 'foo');
