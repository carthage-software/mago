<?php

declare(strict_types=1);

final class ClassesCtorNamedArgs
{
    public function __construct(public string $name, public int $age)
    {
    }
}

$obj = new ClassesCtorNamedArgs(age: 21, name: 'm');
echo $obj->name;
echo $obj->age;
