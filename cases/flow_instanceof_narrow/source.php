<?php

declare(strict_types=1);

interface Animal
{
    public function name(): string;
}

final class Dog implements Animal
{
    public function name(): string
    {
        return 'dog';
    }

    public function bark(): string
    {
        return 'woof';
    }
}

final class Cat implements Animal
{
    public function name(): string
    {
        return 'cat';
    }
}

function flow_instanceof_narrow(Animal $a): string
{
    if ($a instanceof Dog) {
        return $a->bark();
    }

    return $a->name();
}
