<?php

class A {}

/**
 * @property-read A|int $x
 * @property int $y
 * @property-write int $z
 */
class T
{
    private A|int $x;

    public function __construct()
    {
        $this->x = 0;
    }

    public function __get(string $name): mixed
    {
        return $this->__get($name);
    }

    public function __set(string $name, mixed $value): void {}
}

function foo(): void
{
    $t = new T();
    $c = $t->x;
    $t->x = 10;
}

function bar(): void
{
    $t = new T();
    $c = $t->y;
    $t->y = 10;
}

function baz(): void
{
    $t = new T();
    $c = $t->z;
    $t->z = 10;
}
