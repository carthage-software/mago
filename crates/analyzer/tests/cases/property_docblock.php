<?php

class A
{
}

/**
 * @property-read A|int $x
 * @property int $y
 * @property-write int $z
 */
class T
{
    private A|int $x;

    public function __construct() {
        $this->x = 0;
    }

    public function __get(string $name): mixed
    {
        return $this->__get($name);
    }

    public function __set(string $name, mixed $value): void
    {
    }
}

function foo(): void
{
    $t = new T();
    $c = $t->x;
    $t->x = 10; // @mago-expect analysis:invalid-property-write
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
    // @mago-expect analysis:impossible-assignment
    $c = $t->z; // @mago-expect analysis:invalid-property-read
    $t->z = 10;
}
