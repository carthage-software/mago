<?php


/**
 * @method        DateTimeImmutable  subYear()
 */
class A
{
    public function __call(string $name, array $parameters): mixed
    {
        return null;
    }
}

$a = new A();
$a->subYear();

/**
 * @method DateTimeImmutable subYear()
 */
class B
{
    public function __call(string $name, array $parameters): mixed
    {
        return null;
    }
}

$b = new B();
$b->subYear();
