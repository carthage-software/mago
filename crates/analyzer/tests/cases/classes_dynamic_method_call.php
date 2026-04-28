<?php

declare(strict_types=1);

final class ClassesDynamicMethod
{
    public function foo(): int
    {
        return 1;
    }

    public function bar(): int
    {
        return 2;
    }
}

function classesDynMethodCall(ClassesDynamicMethod $obj, string $name): mixed
{
    /** @mago-expect analysis:string-member-selector */
    return $obj->{$name}();
}

$_ = classesDynMethodCall(new ClassesDynamicMethod(), 'foo');
